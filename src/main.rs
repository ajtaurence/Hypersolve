use hypersolve::{
    cubiecube::cubiecube::HYPERSOLVE_TWISTS,
    node_cube::node::Phase1Node,
    prune::{gen_pruning_table, HashMapPruningTable},
};
use itertools::Itertools;

fn main() {
    println!(
        "{:?}",
        HYPERSOLVE_TWISTS.iter().map(|t| t.face).collect_vec()
    );
    let table = gen_pruning_table::<HashMapPruningTable, Phase1Node>(2);
    println!("{:?}", table.data.len());
    println!("{:?}", table.max_depth);
}
