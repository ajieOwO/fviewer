use std::{
    cell::{RefCell, RefMut},
    error::Error,
    fmt, fs,
    path::PathBuf,
    rc::{Rc, Weak},
};

/**
### 文件类型
包含文件夹、普通文件、软连接和其它类型
*/
#[derive(Debug)]
enum FileType {
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
    path: PathBuf,
    /**
    ### 文件类型
    */
    file_type: FileType,
    /**
    ### 父文件夹
    不持有所有权
    */
    parent: Weak<RefCell<Files>>,
    /**
    ### 子文件列表
    持有所有权
     */
    child: Vec<Rc<RefCell<Files>>>,
    /**
    ### 出现的错误
     */
    err: Option<Box<dyn Error>>,
}

// impl fmt::Display for Files {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         if f.alternate() {
//             write!(
//                 f,
//                 "文件: {{路径：{}， 文件类型：{}，子文件：{}，错误：{}}}",
//                 match self.path.to_str() {
//                     Some(str) => str,
//                     None => "[空]",
//                 },
//                 match self.file_type {
//                     FileType::Directory => "文件夹",
//                     FileType::File => "文件",
//                     _ => "其它",
//                 },
//             )
//         } else {
//             write!(f, "Files")
//         }
//     }
// }

impl Files {
    fn new(path: PathBuf) -> Self {
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
}

pub fn scan_files(path: &str, deep: u32) -> Files {
    let mut index_stack: Vec<usize> = Vec::new();
    let root: Rc<RefCell<Files>> = Rc::new(RefCell::new(Files::new(PathBuf::from(path))));
    let mut current_file: Weak<RefCell<Files>> = Rc::downgrade(&root); // 此指针不可能为空

    loop {
        // 索引栈不为空时，尝试指向下一个元素；找不到就指向父元素
        current_file = update_file_ptr(&mut index_stack, deep, current_file);
        // 尝试持有文件的指针
        let current = current_file.upgrade();
        if current.is_none() {
            break;
        }
        let current = current.unwrap();
        let mut current = current.borrow_mut();

        match &current.file_type {
            // 为文件夹时，遍历扫描文件夹下文件
            FileType::Directory => {
                let read_dir = fs::read_dir(&current.path);

                // 遍历处理文件夹下的文件
                for file_name in read_dir.unwrap() {
                    // 文件名读取失败时跳过
                    if file_name.is_err() {
                        continue;
                    }
                    // 拼接完整路径
                    let file_path = file_name.unwrap().path();
                    let mut file: Files = Files::new(file_path);
                    file.parent = current_file.clone(); // 设置父文件夹指针
                    current.child.push(Rc::new(RefCell::new(file)));
                }
            }
            _ => {
                println!("_");
                // if index_stack.is_empty() {
                //     break;
                // }
                // let index = index_stack.pop().unwrap();
                // let parent_file = current.parent.upgrade();
                // if parent_file.is_none() {
                //     ()
                // }
                // let parent = parent_file.unwrap();

                // for (sub_index, sub_file) in parent.borrow().child[index..].iter().enumerate() {
                //     if let FileType::Directory = sub_file.borrow().file_type {
                //         current_file = Rc::downgrade(&sub_file);
                //         index_stack.push(index + sub_index);
                //         break;
                //     }
                // }
            }
        }
    }

    return Rc::try_unwrap(root).unwrap().into_inner();
}

/**
### 更新文件指针
### 参数
- `index_stack`: 索引栈
- `deep`: 遍历深度
- `current_file`：当前文件指针
### 返回值
- 更新后的文件指针
*/
fn update_file_ptr(
    index_stack: &mut Vec<usize>,
    deep: u32,
    current_file: Weak<RefCell<Files>>,
) -> Weak<RefCell<Files>> {
    // 索引栈为空时，解析根文件
    if index_stack.is_empty() {
        index_stack.push(0);
        return current_file;
    }

    let mut result = current_file.clone();

    // 获取当前元素的引用
    let current = current_file.upgrade();
    let current = current.unwrap();
    let current = current.borrow();

    // 获取子元素列表的迭代器
    let iter = current.child.iter();
    let mut has_sub = false;

    // 索引栈深度小于目标值，才向子元素索引
    if index_stack.len() < deep as usize {
        // 遍历迭代器
        for (i, sub_file) in iter.enumerate() {
            // 尝试找到一个类型为文件夹的子元素
            if let FileType::Directory = sub_file.borrow().file_type {
                index_stack.push(i);
                has_sub = true;
                // 指针指向文件夹类型子元素
                result = Rc::downgrade(sub_file);
                break;
            }
        }
    }

    // 当没有类型为文件夹的子元素时，向后匹配兄弟元素
    if !has_sub {
        // 获取当前元素在父元素的索引
        if index_stack.is_empty() {
            return Weak::new();
        }
        let index = index_stack.last_mut().unwrap();

        // 获取父元素的引用
        let parent = &current.parent;
        let parent = parent.upgrade();
        if parent.is_none() {
            return Weak::new();
        }
        let parent = parent.unwrap();
        let parent = parent.borrow();

        // 获取后方兄弟元素的迭代器
        let iter = parent.child[*index + 1..].iter();
        // 尝试找到一个类型为文件夹的兄弟元素
        for (i, sub_file) in iter.enumerate() {
            if let FileType::Directory = sub_file.borrow().file_type {
                *index += i;
                has_sub = true;
                // 指针指向文件夹类型弟元素
                result = Rc::downgrade(sub_file);
                break;
            }
        }
    }

    // 当没有类型为文件夹的兄弟元素时，返回空指针
    if !has_sub {
        return Weak::new();
    }
    return result;
}
