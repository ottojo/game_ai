use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use graphviz_rust::dot_structures::Graph;
use rand::seq::SliceRandom;

use game_ai::{GameAi, GameRules, GameStateTrait, Rewards};

use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use std::time::{Duration, Instant};

use itertools::Itertools;

static mut NEXT_ID: i32 = 1;

#[derive(Clone)]
pub struct GenericMonteCarloTreeSearchAi<Rules: GameRules> {
    // TODO: should MCTS be a trait???

    // TODO: Persist tree
    // tree: Rc<RefCell<Tree>>,
    search_duration: Duration,
    rules: Rules,
}

impl<Rules: GameRules> GenericMonteCarloTreeSearchAi<Rules> {
    pub fn new(duration: Duration, rules: Rules) -> GenericMonteCarloTreeSearchAi<Rules> {
        GenericMonteCarloTreeSearchAi {
            // tree
            search_duration: duration,
            rules,
        }
    }
}

pub struct DebugData {
    pub dot_graph: Graph,
}

impl<Rules: GameRules> GenericMonteCarloTreeSearchAi<Rules> {
    // TODO: No need to take ownership of state if it already exists in tree. Maybe require Clone for state?
    pub fn best_action(&self, state: Rules::State) -> (Rules::Action, DebugData) {
        let tree: Rc<RefCell<Tree<Rules>>> = Rc::new(RefCell::new(Tree {
            state,
            rewards: Rewards {
                player_0: 0.0,
                player_1: 0.0,
            },
            playouts_from_here: 0.0,
            children: HashMap::new(),
            parent: None,
            id: 0,
        }));

        //for _i in 0..100000 {
        //    do_mcts_iteration(Rc::clone(&tree));
        //}

        let start = Instant::now();
        while start.elapsed() < self.search_duration {
            do_mcts_iteration(Rc::clone(&tree));
        }

        let best_move = tree.borrow().select_best_next_move();
        (
            best_move,
            DebugData {
                dot_graph: print_dot(Rc::clone(&tree)),
            },
        )
    }
}

impl<Rules: GameRules> GameAi<Rules> for GenericMonteCarloTreeSearchAi<Rules> {
    fn determine_next_move(&mut self, gamestate: &Rules::State) -> Rules::Action {
        self.best_action(gamestate.clone()).0
    }

    fn name(&self) -> String {
        "Generic Monte Carlo tree search".into()
    }
}

#[allow(unused)]
#[derive(Debug)]
struct Tree<Rules: GameRules> {
    state: Rules::State,
    rewards: Rewards,
    playouts_from_here: f32,
    children: HashMap<Rules::Action, Rc<RefCell<Tree<Rules>>>>,
    parent: Option<Weak<RefCell<Tree<Rules>>>>,
    id: i32,
}

impl<Rules: GameRules> Tree<Rules> {
    /// Returns true if a child exists for every possible move.
    #[allow(unused)]
    fn is_fully_explored(&self) -> bool {
        let all_moves = self.state.get_actions();
        self.children.len() == all_moves.len()
    }

    /// Selects child with max UCB1 score
    fn select_best_next_move(&self) -> Rules::Action {
        assert!(!self.children.is_empty());
        let c = 1.0;
        let parent_playouts = self.playouts_from_here;
        let ucb1 = |child: &Rc<RefCell<Tree<Rules>>>| -> f32 {
            child.borrow().rewards.for_player(&self.state.next_player())
                / child.borrow().playouts_from_here
                + c * (2.0 * parent_playouts.ln() / child.borrow().playouts_from_here).sqrt()
        };
        self.children
            .iter()
            // Child nodes with max UCB1 value
            .max_set_by(|(_move_1, n1), (_move_2, n2)| ucb1(n1).total_cmp(&ucb1(n2)))
            // Random tiebreaker
            .choose(&mut rand::thread_rng())
            .unwrap()
            .0
            .clone()
    }
}

fn do_mcts_iteration<Rules: GameRules>(tree: Rc<RefCell<Tree<Rules>>>) {
    //let selected_node = selection(tree);
    let selected_node = recursive_selection(tree);
    let new_child = expansion(selected_node);
    let result = rollout(Rc::clone(&new_child));
    new_child.borrow_mut().rewards += &result;
    new_child.borrow_mut().playouts_from_here += 1.0;
    backup(Rc::clone(&new_child), result);
}

#[allow(unused)]
fn selection<Rules: GameRules>(mut tree: Rc<RefCell<Tree<Rules>>>) -> Rc<RefCell<Tree<Rules>>> {
    /*
       The selection phase traverses the tree level by level, each time
       selecting a node based on stored statistics like ”number of visits” or ”total
       reward”. The rule by which the algorithm selects is called the tree policy.
       Selection stops when a node is reached that is not fully explored yet, i.e.
       not all possible moves have been expanded to new nodes yet.
    */

    loop {
        if !tree.borrow().is_fully_explored() {
            return tree;
        }
        if tree.borrow().state.is_final() {
            //println!("Selection reached final state");
            // TODO: return none
            return tree;
        }

        // Choose child with max UCB1 score
        let best_move = tree.borrow().select_best_next_move();
        let max_ucb1_child = Rc::clone(tree.borrow().children.get(&best_move).unwrap());
        tree = max_ucb1_child;
    }
}

fn recursive_selection<Rules: GameRules>(
    tree: Rc<RefCell<Tree<Rules>>>,
) -> Rc<RefCell<Tree<Rules>>> {
    if tree.borrow().state.is_final() {
        // Final state can not be expanded
        return tree;
    }

    if !tree.borrow().is_fully_explored() {
        return tree;
    }

    // TODO: It seems this just goes in depth every time? and finds no loss at all in first round?
    // Never chooses entirely unexplored child!

    // Not final, fully explored -> has children
    let best_move = tree.borrow().select_best_next_move();
    let max_ucb1_child = Rc::clone(tree.borrow().children.get(&best_move).unwrap());
    recursive_selection(max_ucb1_child)
}

fn expansion<Rules: GameRules>(tree: Rc<RefCell<Tree<Rules>>>) -> Rc<RefCell<Tree<Rules>>> {
    /*
       The expansion step consists of adding one or multiple new
       child nodes to the final selected node.
    */

    //assert!(!tree.borrow().state.is_final());
    if tree.borrow().state.is_final(){
        return tree;
    }


    let mut possible_moves = tree.borrow().state.get_actions();
    possible_moves.retain(|m| !tree.borrow().children.contains_key(m));

    /*
    if possible_moves.is_empty(){
        //println!("Tries to expand node with no possible moves");
        return tree;
    }
    */

    assert!(!possible_moves.is_empty()); // TODO: This fails
    let random_new_move: Rules::Action = possible_moves
        .choose(&mut rand::thread_rng())
        .unwrap()
        .clone();

    let new_state = Rules::play(&tree.borrow().state, &random_new_move);

    let child = Rc::new(RefCell::new(Tree {
        state: new_state,
        rewards: Rewards {
            player_0: 0.0,
            player_1: 0.0,
        },
        playouts_from_here: 0.0,
        children: HashMap::new(),
        parent: Some(Rc::downgrade(&tree)),
        id: unsafe { NEXT_ID },
    }));

    unsafe { NEXT_ID += 1 };

    tree.borrow_mut()
        .children
        .insert(random_new_move, Rc::clone(&child));

    child
}

fn random_rollout<Rules: GameRules>(initial_state: &Rules::State) -> Rewards {
    let mut state: Rules::State = initial_state.clone();
    while !state.is_final() {
        let all_possible_actions = state.get_actions();
        let random_action = all_possible_actions
            .choose(&mut rand::thread_rng())
            .expect("Rollout failed, no actions possible!");
        state = Rules::play(&state, random_action);
    }

    state.reward()
}

fn rollout<Rules: GameRules>(tree: Rc<RefCell<Tree<Rules>>>) -> Rewards {
    return random_rollout::<Rules>(&tree.borrow().state);
}

fn backup<Rules: GameRules>(child: Rc<RefCell<Tree<Rules>>>, result: Rewards) {
    let mut parent = child.as_ref().borrow().parent.clone();
    loop {
        if parent.is_none() {
            return;
        }

        let parent_rc: Rc<_> = parent.unwrap().upgrade().unwrap();

        parent_rc.borrow_mut().playouts_from_here += 1.0;
        parent_rc.borrow_mut().rewards += &result;

        parent = parent_rc.borrow().parent.clone();
    }
}

fn print_dot<Rules: GameRules>(tree: Rc<RefCell<Tree<Rules>>>) -> Graph {
    let mut layer = vec![tree];
    let mut layer_index = 0;

    let mut graph = graph!(strict di "mcts_tree");

    while !layer.is_empty() {
        let mut subgraph = subgraph!(format!("depth_{}", layer_index));
        subgraph.stmts.push(attr!("rank", "same").into());

        for node in layer.iter() {
            let node_name = format!("state_{}", node.borrow().id);
            let node_tooltip = format!("\"{:?}\"", node.borrow().state);
            let mut dot_node = node!(node_name; attr!("tooltip", node_tooltip));

            let parent_playouts: Option<f32> = node
                .borrow()
                .parent
                .as_ref()
                .map(|parent| parent.upgrade().unwrap().borrow().playouts_from_here);
            let c = 1.0;
            let incoming_player = node.borrow().state.incoming_player();
            let ucb1 = parent_playouts.map(|parent_playouts| {
                node.borrow().rewards.for_player(&incoming_player)
                    / node.borrow().playouts_from_here
                    + c * (2.0 * parent_playouts.ln() / node.borrow().playouts_from_here).sqrt()
            });

            let node_label = format!(
                "\"{}Win/{}Sim ({:.1})\"",
                node.borrow().rewards.player_0,
                node.borrow().playouts_from_here,
                ucb1.unwrap_or(-1.0)
            );
            dot_node.attributes.push(attr!("label", node_label));

            if node.borrow().is_fully_explored() {
                dot_node.attributes.push(attr!("penwidth", 3));
            }

            if node.borrow().state.is_final() {
                let reward = node.borrow().state.reward();
                dot_node.attributes.push(attr!("style", "filled"));

                if reward.player_0 > reward.player_1 {
                    dot_node.attributes.push(attr!("fillcolor", "greenyellow"));
                } else if reward.player_1 > reward.player_0 {
                    dot_node.attributes.push(attr!("fillcolor", "red"));
                } else {
                    dot_node.attributes.push(attr!("fillcolor", "yellow"));
                }
            }
            subgraph.stmts.push(dot_node.into());

            for (action, child) in node.borrow().children.iter() {
                let child_name = format!("state_{}", child.borrow().id);
                let tooltip = format!("\"{:?}\"", action);
                graph.add_stmt(
                    edge!(node_id!(node_name) => node_id!(child_name); attr!("tooltip", tooltip))
                        .into(),
                );
            }
        }

        let mut next_layer = vec![];
        for node in layer {
            next_layer.extend(node.borrow().children.values().map(Rc::clone))
        }

        graph.add_stmt(subgraph.into());

        layer_index += 1;
        layer = next_layer;
    }

    graph
}