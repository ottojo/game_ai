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

#[derive(Clone)]
pub struct GenericMonteCarloTreeSearchAi<Rules: GameRules> {
    // TODO: should MCTS be a trait???

    // TODO: Persist tree
    // tree: Rc<RefCell<Tree>>,
    stop_condition: StopCondition,
    last_tree: Rc<RefCell<Tree<Rules>>>,
    next_id: i32,
}

impl<Rules: GameRules> GenericMonteCarloTreeSearchAi<Rules> {
    pub fn new(stop_condition: StopCondition) -> GenericMonteCarloTreeSearchAi<Rules> {
        GenericMonteCarloTreeSearchAi {
            stop_condition,
            last_tree: Default::default(),
            next_id: 1,
        }
    }
}

#[derive(Clone)]
pub enum StopCondition {
    Iterations(usize),
    Time(Duration),
}

impl<Rules: GameRules> GenericMonteCarloTreeSearchAi<Rules> {
    fn do_mcts_iteration(&mut self, tree: Rc<RefCell<Tree<Rules>>>) {
        let selected_node = selection(tree);
        let new_child = self.expansion(selected_node);
        let result = rollout(Rc::clone(&new_child));
        new_child.borrow_mut().rewards += &result;
        new_child.borrow_mut().playouts_from_here += 1.0;
        backup(Rc::clone(&new_child), result);
    }

    fn expansion(&mut self, tree: Rc<RefCell<Tree<Rules>>>) -> Rc<RefCell<Tree<Rules>>> {
        if tree.borrow().state.is_final() {
            // Can not expand a final state, so just do trivial rollout and backpropagation
            return tree;
        }

        // Choose random move that has not been explored yet
        let mut possible_moves = tree.borrow().state.get_actions();
        possible_moves.retain(|m| !tree.borrow().children.contains_key(m));
        assert!(!possible_moves.is_empty());

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
            id: self.next_id,
            fully_explored_cache: false,
        }));

        self.next_id += 1;

        tree.borrow_mut()
            .children
            .insert(random_new_move, Rc::clone(&child));

        child
    }
}

impl<Rules: GameRules> GameAi<Rules> for GenericMonteCarloTreeSearchAi<Rules> {
    fn determine_next_move(&mut self, state: &Rules::State) -> Rules::Action {
        self.last_tree = Rc::new(RefCell::new(Tree {
            state: state.clone(),
            rewards: Rewards {
                player_0: 0.0,
                player_1: 0.0,
            },
            playouts_from_here: 0.0,
            children: HashMap::new(),
            parent: None,
            id: 0,
            fully_explored_cache: false,
        }));

        match self.stop_condition {
            StopCondition::Iterations(iterations) => {
                for _i in 0..iterations {
                    self.do_mcts_iteration(Rc::clone(&self.last_tree));
                }
            }
            StopCondition::Time(duration) => {
                let start = Instant::now();
                while start.elapsed() < duration {
                    self.do_mcts_iteration(Rc::clone(&self.last_tree));
                }
            }
        }

        let best_move = self.last_tree.borrow().select_best_next_move();

        best_move
    }

    fn name(&self) -> String {
        "Generic Monte Carlo tree search".into()
    }
}

#[derive(Debug, Clone)]
struct Tree<Rules: GameRules> {
    state: Rules::State,
    rewards: Rewards,
    playouts_from_here: f32,
    children: HashMap<Rules::Action, Rc<RefCell<Tree<Rules>>>>,
    parent: Option<Weak<RefCell<Tree<Rules>>>>,
    id: i32,
    fully_explored_cache: bool,
}

impl<Rules: GameRules> Default for Tree<Rules> {
    fn default() -> Self {
        Self {
            state: Default::default(),
            rewards: Default::default(),
            playouts_from_here: Default::default(),
            children: Default::default(),
            parent: Default::default(),
            id: Default::default(),
            fully_explored_cache: false,
        }
    }
}

impl<Rules: GameRules> Tree<Rules> {
    /// Returns true if a child exists for every possible move.
    fn is_fully_explored(&mut self) -> bool {
        if self.fully_explored_cache {
            return true;
        }

        let all_moves = self.state.get_actions();
        let explored = self.children.len() == all_moves.len();
        self.fully_explored_cache = explored;
        explored
    }

    /// Selects child with max UCB1 score
    fn select_best_next_move(&self) -> Rules::Action {
        assert!(!self.children.is_empty());
        let c = 1.0;
        let parent_playouts = self.playouts_from_here;
        let player = self.state.next_player();
        let ucb1 = |child: &Rc<RefCell<Tree<Rules>>>| -> f32 {
            let child_playouts = child.borrow().playouts_from_here;
            child.borrow().rewards.for_player(&player) / child_playouts
                + c * (2.0 * parent_playouts.ln() / child_playouts).sqrt()
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

fn selection<Rules: GameRules>(mut tree: Rc<RefCell<Tree<Rules>>>) -> Rc<RefCell<Tree<Rules>>> {
    // Select node to expand by tree policy, in this case recursively max UCB1 value
    loop {
        if tree.borrow().state.is_final() {
            // Final state can not be expanded
            return tree;
        }
        if !tree.borrow_mut().is_fully_explored() {
            return tree;
        }

        // Choose child with max UCB1 score
        let best_move = tree.borrow().select_best_next_move();
        let max_ucb1_child = Rc::clone(tree.borrow().children.get(&best_move).unwrap());
        tree = max_ucb1_child;
    }
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

impl<Rules: GameRules> GenericMonteCarloTreeSearchAi<Rules> {
    /// Get graphviz representation of last search tree
    pub fn get_last_graphviz(&self) -> Graph {
        let mut layer = vec![Rc::clone(&self.last_tree)];
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

                if node.borrow_mut().is_fully_explored() {
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
}
