use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::scanner::files::{FileType, Files};


/// ### 更新文件指针
/// ### 参数
/// - `index_stack`: 索引栈
/// - `deep`: 遍历深度
/// - `current_file`：当前文件指针
/// ### 返回值
/// - 更新后的文件指针
pub fn traverse(
    index_stack: &mut Vec<usize>,
    deep: usize,
    current_file: Weak<RefCell<Files>>,
) -> Weak<RefCell<Files>> {
    // 索引栈为空时，解析根文件
    if index_stack.is_empty() {
        index_stack.push(0);
        return current_file;
    }

    let mut result = Weak::new();
    let mut current_file = current_file;
    let mut no_child = false;

    loop {
        // 获取当前元素的引用
        let current = current_file.upgrade();
        let current = current.unwrap();
        let current = current.borrow();

        // 获取子元素列表的迭代器
        let iter = current.child.iter();
        let mut has_sub = false; // 是否拥有子元素

        if no_child {
            index_stack.pop();
        } else {
            // 索引栈深度小于目标值，才向子元素索引
            if index_stack.len() < deep {
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
                    *index += i + 1;
                    has_sub = true;
                    // 指针指向文件夹类型弟元素
                    result = Rc::downgrade(sub_file);
                    break;
                }
            }
        }

        // 当没有类型为文件夹的兄弟元素时，跳转到父文件夹
        if !has_sub {
            current_file = current.parent.clone();
            no_child = true;
            continue;
        }
        break;
    }
    return result;
}
