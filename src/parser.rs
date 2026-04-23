use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::types::CustomChange;

pub fn read_need_change(path: &str) -> Vec<CustomChange> {
    let file = File::open(path).expect("无法打开needChange.dmp");
    let reader = BufReader::new(file);
    let mut changes = Vec::new();
    
    for (line_num, line) in reader.lines().enumerate() {
        let line = line.expect("读取行失败");
        if line.trim().is_empty() {
            continue;
        }
        
        match CustomChange::from_line(&line) {
            Some(change) => {
                changes.push(change);
            }
            None => {
                eprintln!("警告: 第{}行解析失败: {}", line_num + 1, line);
            }
        }
    }
    
    changes
}

pub fn read_taxid_list(path: &str) -> Vec<u64> {
    let file = File::open(path).expect("无法打开InnoDB.taxidList");
    let reader = BufReader::new(file);
    let mut taxids = Vec::new();
    
    for line in reader.lines() {
        let line = line.expect("读取行失败");
        let taxid: u64 = line.trim().parse().expect("taxid解析失败");
        taxids.push(taxid);
    }
    
    taxids
}

pub fn parse_nodes(path: &str) -> (Vec<String>, HashMap<u64, u64>, HashMap<u64, Vec<String>>) {
    use std::io::BufRead;
    
    let file = File::open(path).expect("无法打开nodes.dmp");
    let reader = BufReader::new(file);
    let mut all_nodes = Vec::new();
    let mut parent_map: HashMap<u64, u64> = HashMap::new();
    let mut node_info_map: HashMap<u64, Vec<String>> = HashMap::new();
    
    for line in reader.lines() {
        let line = line.expect("读取行失败");
        let line_owned = line.clone();
        
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 5 {
            continue;
        }
        
        let taxid: u64 = fields[0].trim().parse().expect("taxid解析失败");
        
        // 第2个字段是parent（索引2，因为格式是: taxid \t | \t parent \t | ...）
        let parent: u64 = if fields.len() > 2 {
            fields[2].trim().parse().expect("parent解析失败")
        } else {
            continue;
        };
        
        parent_map.insert(taxid, parent);
        
        // 存储前5个字段
        let first_five: Vec<String> = fields[..5.min(fields.len())]
            .iter()
            .map(|s| s.to_string())
            .collect();
        node_info_map.insert(taxid, first_five);
        
        all_nodes.push(line_owned);
    }
    
    (all_nodes, parent_map, node_info_map)
}

pub fn parse_names(path: &str) -> (Vec<String>, HashMap<u64, Vec<String>>) {
    use std::io::BufRead;
    
    let file = File::open(path).expect("无法打开names.dmp");
    let reader = BufReader::new(file);
    let mut all_names = Vec::new();
    let mut name_map: HashMap<u64, Vec<String>> = HashMap::new();
    
    for line in reader.lines() {
        let line = line.expect("读取行失败");
        let line_owned = line.clone();
        
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.is_empty() {
            continue;
        }
        
        // 只处理科学名记录（像Python一样）
        if fields.len() >= 7 && fields[6].trim() == "scientific name" {
            let taxid: u64 = fields[0].trim().parse().unwrap_or(0);
            if taxid > 0 {
                // 存储名称字段（像Python的namesdict，存储字段5-13）
                // 但我们需要存储整个名称记录
                name_map.insert(taxid, fields.iter().map(|s| s.to_string()).collect());
            }
        }
        all_names.push(line_owned);
    }
    
    (all_names, name_map)
}
