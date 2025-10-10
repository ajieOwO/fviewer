use std::{cell::RefCell, error::Error, fs, path::PathBuf, rc::{Rc, Weak}};

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
}
