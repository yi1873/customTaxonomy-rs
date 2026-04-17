use std::collections::{HashMap, HashSet};

use clap::Parser;

use custom_taxonomy_rs::parser::{read_need_change, read_taxid_list, parse_nodes, parse_names};
use custom_taxonomy_rs::writer::{write_filtered_nodes, write_filtered_names};
use custom_taxonomy_rs::types::CustomChange;

#[derive(Parser, Debug)]
#[command(name = "custom-taxonomy-rs")]
#[command(about = "Filter taxonomy database based on target taxids and custom changes")]
struct Args {
    /// Working directory (default: current directory)
    #[arg(long, default_value = ".")]
    work_dir: String,

    /// Path to taxid list file (required)
    #[arg(long)]
    taxid_list: String,

    /// Path to needChange.dmp file (required)
    #[arg(long)]
    need_change: String,

    /// Path to names.dmp file
    #[arg(long)]
    names_file: Option<String>,

    /// Path to nodes.dmp file
    #[arg(long)]
    nodes_file: Option<String>,

    /// Output names.dmp file
    #[arg(long)]
    output_names: Option<String>,

    /// Output nodes.dmp file
    #[arg(long)]
    output_nodes: Option<String>,
}

fn main() {
    let args = Args::parse();

    // 构建文件路径
    let work_dir = &args.work_dir;
    let names_file = args.names_file.unwrap_or_else(|| format!("{}/data/names.dmp", work_dir));
    let nodes_file = args.nodes_file.unwrap_or_else(|| format!("{}/data/nodes.dmp", work_dir));
    let taxid_list_file = &args.taxid_list;
    let need_change_file = &args.need_change;
    let output_names = args.output_names.unwrap_or_else(|| format!("{}/names_filtered.dmp", work_dir));
    let output_nodes = args.output_nodes.unwrap_or_else(|| format!("{}/nodes_filtered.dmp", work_dir));

    println!("工作目录: {}", work_dir);
    println!("读取自定义修改关系: {}", need_change_file);
    let custom_changes = read_need_change(need_change_file);
    println!("  加载了 {} 条修改规则", custom_changes.len());
    for change in &custom_changes {
        println!("    规则: taxid={}, new_parent={}, rank={:?}", 
                 change.taxid, change.new_parent, change.rank);
    }
    
    println!("读取目标taxid列表: {}", taxid_list_file);
    let target_taxids = read_taxid_list(taxid_list_file);
    println!("  目标taxid数量: {}", target_taxids.len());
    
    println!("解析原始nodes.dmp: {}", nodes_file);
    let (all_nodes, parent_map, node_info_map): (Vec<String>, HashMap<u64, u64>, HashMap<u64, Vec<String>>) = parse_nodes(&nodes_file);
    println!("  总节点数: {}", all_nodes.len());
    
    println!("解析原始names.dmp: {}", names_file);
    let (all_names, name_map): (Vec<String>, HashMap<u64, Vec<String>>) = parse_names(&names_file);
    println!("  科学名记录数: {}", name_map.len());
    
    // 构建合并的父节点映射（自定义修改覆盖原始关系）
    let mut merged_parent_map: HashMap<u64, u64> = parent_map.clone();
    let mut custom_taxids: HashSet<u64> = HashSet::new();
    
    for change in &custom_changes {
        merged_parent_map.insert(change.taxid, change.new_parent);
        custom_taxids.insert(change.taxid);
    }
    
    // 初始需要包含的taxid：目标taxid + 自定义修改中的taxid
    let mut needed_taxids: HashSet<u64> = HashSet::new();
    
    for taxid in &target_taxids {
        needed_taxids.insert(*taxid);
    }
    
    for change in &custom_changes {
        needed_taxids.insert(change.taxid);
    }
    
    println!("追溯lineage（使用合并映射）...");
    // 为所有当前需要的taxid追溯完整lineage
    let initial_taxids: Vec<u64> = needed_taxids.iter().copied().collect();
    for &taxid in &initial_taxids {
        collect_lineage(taxid, &merged_parent_map, &mut needed_taxids);
    }
    
    // 还需要追溯自定义修改中父节点的lineage（确保父节点的祖先也被包含）
    for change in &custom_changes {
        if !custom_taxids.contains(&change.new_parent) {
            // 父节点不是自定义taxid，需要追溯其lineage
            collect_lineage(change.new_parent, &merged_parent_map, &mut needed_taxids);
        }
    }
    
    println!("  追溯后总taxid数量: {}", needed_taxids.len());
    
    println!("输出筛选结果...");
    write_filtered_nodes(&output_nodes, &all_nodes, &needed_taxids, &custom_changes)
        .expect("写入nodes文件失败");
    write_filtered_names(&output_names, &all_names, &needed_taxids, &custom_changes, &name_map)
        .expect("写入names文件失败");
    
    println!("完成!");
    println!("  输出names: {}", output_names);
    println!("  输出nodes: {}", output_nodes);
}

fn collect_lineage(taxid: u64, parent_map: &HashMap<u64, u64>, needed: &mut HashSet<u64>) {
    let mut current = taxid;
    let mut visited = HashSet::new();
    
    loop {
        if visited.contains(&current) {
            break;
        }
        visited.insert(current);
        
        needed.insert(current);
        
        match parent_map.get(&current) {
            Some(&parent) if parent != current => {
                current = parent;
            }
            _ => break,
        }
    }
}
