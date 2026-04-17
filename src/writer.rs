use std::fs::File;
use std::io::Write;
use std::collections::{HashMap, HashSet};

use crate::types::CustomChange;

pub fn write_filtered_nodes(
    output_path: &str,
    all_nodes: &[String],
    needed: &HashSet<u64>,
    custom_changes: &[CustomChange],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(output_path)?;
    
    // 创建taxid到CustomChange的映射
    let custom_map: HashMap<u64, &CustomChange> = custom_changes.iter()
        .map(|c| (c.taxid, c))
        .collect();
    
    // 创建taxid到原始完整行的映射
    let mut original_line_map: HashMap<u64, &String> = HashMap::new();
    for line in all_nodes {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.is_empty() {
            continue;
        }
        if let Ok(taxid) = fields[0].trim().parse::<u64>() {
            original_line_map.insert(taxid, line);
        }
    }
    
    // 对于每个需要的taxid，输出节点记录
    for &taxid in needed {
        if let Some(change) = custom_map.get(&taxid) {
            if original_line_map.contains_key(&taxid) {
                // 已存在的taxid：从原始nodes.dmp获取完整行，只更新parent和rank
                // 找到原始行
                let original_line = original_line_map.get(&taxid).unwrap();
                let fields: Vec<&str> = original_line.split('\t').collect();
                // 重建行，更新字段2(parent)和字段4(rank)
                let mut new_fields: Vec<String> = fields.iter().map(|s| s.to_string()).collect();
                if new_fields.len() >= 3 {
                    new_fields[2] = change.new_parent.to_string();
                }
                if new_fields.len() >= 5 {
                    new_fields[4] = change.rank.clone();
                }
                writeln!(file, "{}", new_fields.join("\t"))?;
            } else {
                // 新增taxid：构建完整的nodes.dmp格式行
                // 标准格式共17个数据列+末尾|
                // 列索引(0-based): 0=taxid, 1=|, 2=parent, 3=|, 4=rank, 5=|,
                // 6=embl_code(空), 7=|, 8=division_id(0), 9=|,
                // 10=inherited_div_flag(1), 11=|, 12=genetic_code_id(11), 13=|,
                // 14=inherited_GC_flag(1), 15=|, 16=mitochondrial_genetic_code_id(0),
                // ...后续列补0和空
                // 
                // 参考原始nodes.dmp格式:
                // taxid \t | \t parent \t | \t rank \t | \t embl \t | \t div_id \t |
                // \t inh_div \t | \t gc_id \t | \t inh_gc \t | \t mito_gc \t |
                // \t inh_mito_gc \t | \t GenBank_hidden \t | \t hidden_subtree \t |
                // \t comments \t | \t plastid_gc \t | \t inh_plastid_gc \t |
                // \t spec_org \t | \t inh_spec_org \t | \t |
                //
                // 用户指定第8-15列填: |  0  |  1  |  11  |  1
                // 即: embl_code=空, division_id=0, inherited_div=1, genetic_code_id=11, inherited_GC=1
                
                let taxid_str = change.taxid.to_string();
                let parent_str = change.new_parent.to_string();
                let rank_str = change.rank.clone();
                
                let new_node_line = format!(
                    "{}\t|\t{}\t|\t{}\t|\t\t|\t0\t|\t1\t|\t11\t|\t1\t|\t0\t|\t0\t|\t0\t|\t\t|\t\t|\t0\t|\t0\t|\t1\t|",
                    taxid_str, parent_str, rank_str
                );
                writeln!(file, "{}", new_node_line)?;
            }
        } else {
            // 非自定义修改的taxid：直接输出原始行
            if let Some(original_line) = original_line_map.get(&taxid) {
                writeln!(file, "{}", original_line)?;
            }
        }
    }
    
    Ok(())
}

pub fn write_filtered_names(
    output_path: &str,
    _all_names: &[String],
    needed: &HashSet<u64>,
    custom_changes: &[CustomChange],
    name_map: &HashMap<u64, Vec<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(output_path)?;
    
    // 创建taxid到CustomChange的映射
    let custom_map: HashMap<u64, &CustomChange> = custom_changes.iter()
        .map(|c| (c.taxid, c))
        .collect();
    
    // 对于每个需要的taxid，输出名称记录
    for &taxid in needed {
        if let Some(change) = custom_map.get(&taxid) {
            // 自定义修改的taxid：使用needChange.dmp中的名称信息
            if change.name_info.len() >= 8 {
                let name_line = change.name_info.join("\t");
                writeln!(file, "{}", name_line)?;
            }
        } else if let Some(name_info) = name_map.get(&taxid) {
            // 非自定义taxid：使用原始科学名记录
            let name_line = name_info.join("\t");
            writeln!(file, "{}", name_line)?;
        }
    }
    
    Ok(())
}
