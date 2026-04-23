#[derive(Debug, Clone)]
pub struct CustomChange {
    pub taxid: u64,
    pub name: Option<String>,
    pub new_parent: u64,
    pub rank: String,
    pub original_line: String,
    pub tab_fields: Vec<String>, // 按制表符分割的字段
    pub node_info: Vec<String>,  // 节点信息：5个字段，格式为 [taxid, "|", parent, "|", rank]
    pub name_info: Vec<String>,  // 名称信息：8个字段，格式为 fields[5..13]
}

impl CustomChange {
    pub fn from_line(line: &str) -> Option<Self> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }
        
        // 按制表符分割字段
        let tab_fields: Vec<String> = line.split('\t').map(|s| s.to_string()).collect();
        if tab_fields.len() != 13 {
            eprintln!("警告: 字段数 {} != 13，跳过: {}", tab_fields.len(), line);
            return None;
        }
        
        // 从制表符分割的字段中解析值
        // taxid是第一个字段
        let taxid_str = tab_fields[0].trim();
        // 名称是第三个字段（索引2）
        let name_str = tab_fields[2].trim();
        // 父节点：尝试索引10，如果失败则尝试11
        let mut parent_idx = 10;
        if parent_idx >= tab_fields.len() || tab_fields[parent_idx].trim() == "|" || tab_fields[parent_idx].trim().is_empty() {
            parent_idx = 11;
        }
        if parent_idx >= tab_fields.len() {
            eprintln!("警告: 父节点索引超出范围: {}", line);
            return None;
        }
        let parent_str = tab_fields[parent_idx].trim();
        // 等级：尝试索引12，如果失败则尝试13
        let mut rank_idx = 12;
        if rank_idx >= tab_fields.len() || tab_fields[rank_idx].trim() == "|" || tab_fields[rank_idx].trim().is_empty() {
            rank_idx = 13;
        }
        if rank_idx >= tab_fields.len() {
            eprintln!("警告: 等级索引超出范围: {}", line);
            return None;
        }
        let rank_str = tab_fields[rank_idx].trim();
        
        let taxid = match taxid_str.parse::<u64>() {
            Ok(id) => id,
            Err(e) => {
                eprintln!("警告: 无法解析taxid '{}': {}", taxid_str, e);
                return None;
            }
        };
        
        let name = if name_str.is_empty() {
            None
        } else {
            Some(name_str.to_string())
        };
        
        let new_parent = match parent_str.parse::<u64>() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("警告: 无法解析parent '{}': {}", parent_str, e);
                return None;
            }
        };
        
        let rank = rank_str.to_string();
        
        // 构建节点信息：5个字段 [taxid, "|", parent, "|", rank]
        let node_info = vec![
            taxid_str.to_string(),
            "|".to_string(),
            parent_str.to_string(),
            "|".to_string(),
            rank_str.to_string(),
        ];
        
        // 构建符合names.dmp格式的名称信息
        // 标准格式: taxid | name | unique_name | type | ...
        // 对于needChange.dmp: 字段1=taxid, 字段3=name, 字段5=unique_name(空), 字段7=type
        let unique_name = if tab_fields.len() > 4 {
            tab_fields[4].trim().to_string()
        } else {
            "".to_string()
        };
        let name_type = if tab_fields.len() > 6 {
            tab_fields[6].trim().to_string()
        } else {
            "scientific name".to_string()
        };
        let name_info = vec![
            taxid_str.to_string(),
            "|".to_string(),
            name_str.to_string(),
            "|".to_string(),
            unique_name,
            "|".to_string(),
            name_type,
            "|".to_string(),
        ];
        
        Some(CustomChange {
            taxid,
            name,
            new_parent,
            rank,
            original_line: line.to_string(),
            tab_fields,
            node_info,
            name_info,
        })
    }
}
