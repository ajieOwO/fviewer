use std::os::unix::fs::PermissionsExt;
pub use std::{
    cell::RefCell,
    error::Error,
    fmt::{self, Result},
    fs,
    path::PathBuf,
    rc::{Rc, Weak},
};

use colored::{ColoredString, Colorize};

/// ### 文件类型
/// 包含文件夹、普通文件、软连接和其它类型

#[derive(Debug)]
pub enum FileType {
    /// ### 文件夹
    /// 包含任意个子文件，同时持有子文件的所有权
    Directory,

    /// ### 普通文件
    File,

    /// ### 软链接
    /// 指向特定文件
    SoftLink(PathBuf),

    ///### 其它类型
    Other,

    /// ### 打开失败类型
    Invalid,
}

#[derive(Debug)]
pub struct Files {
    ///### 文件名
    pub path: PathBuf,

    ///### 文件类型
    pub file_type: FileType,

    /// ### 文件类型(mode)
    pub mode_str: String,

    /// ### 父文件夹
    /// 不持有所有权
    pub parent: Weak<RefCell<Files>>,

    ///### 子文件列表
    /// 持有所有权
    pub child: Vec<Rc<RefCell<Files>>>,

    ///### 出现的错误
    err: Option<Box<dyn Error>>,
}

impl Files {
    pub fn new(path: PathBuf) -> Self {
        let metadata = fs::metadata(&path);
        let file_type = {
            if metadata.is_err() {
                FileType::Invalid
            } else {
                let link_metadata = fs::symlink_metadata(&path);
                let metadata = metadata.as_ref().unwrap();
                if link_metadata
                    .map(|meta| meta.file_type().is_symlink())
                    .unwrap_or(false)
                {
                    let target = fs::read_link(&path).unwrap();
                    FileType::SoftLink(target)
                } else {
                    match () {
                        _ if metadata.is_file() => FileType::File,
                        _ if metadata.is_dir() => FileType::Directory,
                        _ => FileType::Other,
                    }
                }
            }
        };
        let mode = metadata
            .as_ref()
            .map(|metadata| metadata.permissions().mode())
            .unwrap_or(0);
        let mode_str: String = String::from(format!(
            "{}{}{}{}",
            match mode >> 12 {
                0o10 => '-', // 普通文件
                0o04 => 'd', // 文件夹
                0o12 => 'l', // 符号链接
                0o02 => 'c', // 字符设备
                0o06 => 'b', // 块设备
                0o01 => 'p', // FIFO管道
                0o14 => 's', // 套接字
                _ => '?',    // 未知类型
            },
            Self::get_permission((mode & 0o700) >> 6),
            Self::get_permission((mode & 0o070) >> 3),
            Self::get_permission(mode & 0o007),
        ));

        Files {
            path,
            mode_str,
            file_type,
            parent: Weak::new(),
            child: Vec::new(),
            err: match metadata {
                Ok(_) => None,
                Err(err) => Some(Box::new(err)),
            },
        }
    }

    pub fn get_colored_name(&self) -> Vec<ColoredString> {
        let file_name = self.path.file_name().unwrap_or(self.path.as_os_str());
        let file_name = file_name.to_str().unwrap();
        let mut result = vec![Self::get_color_str(&self.file_type, file_name)];
        if let FileType::SoftLink(target) = &self.file_type {
            let target_file = Files::new(target.clone());
            let target_path = target.to_str().unwrap();
            result.push(" -> ".bold());
            result.push(Self::get_color_str(&target_file.file_type, target_path));
        }

        return result;
    }

    fn get_color_str(file_type: &FileType, str: &str) -> ColoredString {
        match file_type {
            FileType::Directory => str.bright_blue().bold(),
            FileType::SoftLink(_) => str.bright_cyan().bold(),
            _ => str.white(),
        }
    }

    fn get_permission(mode: u32) -> String {
        format!(
            "{}{}{}",
            if mode & 0b100 != 0 { 'r' } else { '-' },
            if mode & 0b100 != 0 { 'w' } else { '-' },
            if mode & 0b100 != 0 { 'x' } else { '-' }
        )
    }
}

pub struct FileWithName<'a>(pub &'a Rc<RefCell<Files>>);
pub struct FileInTree<'a>(pub &'a Rc<RefCell<Files>>);

impl fmt::Display for FileWithName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        for file in &self.0.borrow().child {
            let file = file.borrow();
            write!(f, "{}  ", file.get_colored_name()[0]).unwrap();
        }
        write!(f, "")
    }
}

impl fmt::Display for FileInTree<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        let current = self.0;
        write!(f, "{} ", current.borrow().mode_str)?;
        for str in current.borrow().get_colored_name() {
            write!(f, "{}", str)?;
        }
        writeln!(f, "")?;

        let mut index_stack: Vec<usize> = Vec::new();
        let mut count: (usize, usize) = (0, 0);
        if current.borrow().child.is_empty() {
            count.1 += 1;
        } else {
            index_stack.push(0);
        }
        let mut current_file = Rc::downgrade(current);
        let mut prefix: Vec<&str> = Vec::new();

        loop {
            if index_stack.is_empty() {
                break;
            }

            let current = current_file.upgrade().unwrap();
            let current = current.borrow();

            // println!("current options is {:?}", current);
            let len = index_stack.len();
            let index = index_stack.last_mut().unwrap();
            let iter = current.child[*index..].iter();
            let num = current.child.len();
            for file in iter {
                let file_ref = file.borrow();
                *index += 1;
                // 输出到控制台
                write!(
                    f,
                    "{}{}",
                    prefix.join(""),
                    if *index < num { "├─" } else { "└─" }
                )?;
                write!(f, "{}", file_ref.mode_str)?;
                for str in file_ref.get_colored_name() {
                    write!(f, " {} ", str)?;
                }
                writeln!(f, "")?;

                // 为文件夹时入栈
                if let FileType::Directory = file_ref.file_type {
                    count.0 += 1;
                    if !file_ref.child.is_empty() {
                        current_file = Rc::downgrade(file);
                        prefix.push(if *index < num { "│ " } else { "  " });
                        index_stack.push(0);

                        break;
                    }
                } else {
                    count.1 += 1;
                }
            }

            // 当索引栈未加深时，弹栈返回上一层文件夹。
            if index_stack.len() == len {
                index_stack.pop();
                current_file = current.parent.clone();
                if !prefix.is_empty() {
                    prefix.remove(prefix.len() - 1);
                }
            }
        }
        println!("共有子文件夹{}个，文件{}个", count.0, count.1);
        return Ok(());
    }
}
