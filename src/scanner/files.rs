pub use std::{
    cell::RefCell,
    error::Error,
    fmt::{self, Result},
    fs,
    path::PathBuf,
    rc::{Rc, Weak},
};

use colored::{ColoredString, Colorize};

/**
### 文件类型
包含文件夹、普通文件、软连接和其它类型
*/
#[derive(Debug)]
pub enum FileType {
    /**
     ### 文件夹
     包含任意个子文件，同时持有子文件的所有权
    */
    Directory,
    /**
     ### 普通文件
    */
    File,
    /**
    ### 软链接
    指向特定文件
    */
    SoftLink(String),
    /**
    ### 其它类型
    */
    Other,
    /**
    ### 打开失败类型
     */
    Invalid,
}

#[derive(Debug)]
pub struct Files {
    /**
    ### 文件名
     */
    pub path: PathBuf,
    /**
    ### 文件类型
    */
    pub file_type: FileType,
    /**
    ### 父文件夹
    不持有所有权
    */
    pub parent: Weak<RefCell<Files>>,
    /**
    ### 子文件列表
    持有所有权
     */
    pub child: Vec<Rc<RefCell<Files>>>,
    /**
    ### 出现的错误
     */
    err: Option<Box<dyn Error>>,
}

impl Files {
    pub fn new(path: PathBuf) -> Self {
        let metadata = fs::metadata(&path);
        let file_type = {
            if metadata.is_err() {
                FileType::Invalid
            } else {
                let metadata = metadata.as_ref().unwrap();
                if metadata.is_file() {
                    FileType::File
                } else if metadata.is_dir() {
                    FileType::Directory
                } else if metadata.is_symlink() {
                    FileType::SoftLink(String::from(""))
                } else {
                    FileType::Other
                }
            }
        };
        Files {
            path,
            file_type,
            parent: Weak::new(),
            child: Vec::new(),
            err: match metadata {
                Ok(_) => None,
                Err(err) => Some(Box::new(err)),
            },
        }
    }

    pub fn get_colored_name(&self) -> ColoredString {
        let file_name = self.path.file_name().unwrap_or(self.path.as_os_str());
        let file_name = file_name.to_str().unwrap();
        match self.file_type {
            FileType::Directory => file_name.bright_blue().bold(),
            _ => file_name.white(),
        }
    }
}

pub struct FileWithName<'a>(pub &'a Rc<RefCell<Files>>);
pub struct FileInTree<'a>(pub &'a Rc<RefCell<Files>>);

impl fmt::Display for FileWithName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        for file in &self.0.borrow().child {
            let file = file.borrow();
            write!(f, "{}  ", file.get_colored_name()).unwrap();
        }
        write!(f, "")
    }
}

impl fmt::Display for FileInTree<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result {
        let current = self.0;
        writeln!(f, "{}", current.borrow().get_colored_name())?;

        if current.borrow().child.is_empty() {
            return Ok(());
        }

        let mut index_stack: Vec<usize> = Vec::new();
        index_stack.push(0);
        // println!("current_file init as {:?}", current_file.upgrade());
        let mut current_file = Rc::downgrade(current);

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
                *index += 1;
                // 输出到控制台
                writeln!(
                    f,
                    " {}{} {}",
                    " ".repeat(2 * (len - 1)),
                    if *index < num { "├─" } else { "└─" },
                    file.borrow().get_colored_name()
                )?;

                // 为文件时入栈
                if let FileType::Directory = file.borrow().file_type
                    && !file.borrow().child.is_empty()
                {
                    current_file = Rc::downgrade(&current.child[0]);
                    index_stack.push(0);
                    break;
                }
            }

            // 当索引栈未加深时，弹栈返回上一层文件夹。
            if index_stack.len() == len {
                index_stack.pop();
                current_file = current.parent.clone();
            }
            break;
        }
        return Ok(());
    }
}
