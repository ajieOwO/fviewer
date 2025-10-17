use std::{
    cell::RefCell,
    fs,
    path::PathBuf,
    rc::{Rc, Weak},
};

use crate::scanner::{
    file_traverse,
    files::{FileType, Files},
};

/**
### 扫描文件
### 参数
- `path`: 目标路径
- `deep`: 遍历深度
- `all`: 是否展示所有文件
### 返回值
- 文件结构树
*/
pub fn scan_files(path: &str, deep: usize, all: bool) -> Rc<RefCell<Files>> {
    let mut index_stack: Vec<usize> = Vec::new();
    let root: Rc<RefCell<Files>> = Rc::new(RefCell::new(Files::new(PathBuf::from(path))));
    let mut current_file: Weak<RefCell<Files>> = Rc::downgrade(&root); // 此指针不可能为空

    loop {
        // 索引栈不为空时，尝试指向下一个元素；找不到就指向父元素
        current_file = file_traverse::traverse(&mut index_stack, deep, current_file);
        // 尝试持有文件的指针
        let current = current_file.upgrade();
        if current.is_none() {
            break;
        }
        let current = current.unwrap();
        let mut current = current.borrow_mut();

        match &current.file_type {
            // 为文件夹时，遍历扫描文件夹下文件
            &FileType::Directory => {
                let read_dir = fs::read_dir(&current.path);

                // 遍历处理文件夹下的文件
                for file_name in read_dir.unwrap() {
                    // 文件名读取失败时跳过
                    if file_name.is_err() {
                        continue;
                    }
                    // 拼接完整路径
                    let file_path = file_name.unwrap().path();
                    
                    if !all
                        && file_path
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .starts_with(".")
                    {
                        // 未选择查看所有文件时，忽略隐藏文件
                        continue;
                    }
                    let mut file: Files = Files::new(file_path);
                    file.parent = current_file.clone(); // 设置父文件夹指针
                    current.child.push(Rc::new(RefCell::new(file)));
                }
                current
                    .child
                    .sort_by(|a, b| a.borrow().path.cmp(&b.borrow().path));
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

    return root;
}
