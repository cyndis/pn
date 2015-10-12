use std::fmt::Debug;
use std::collections::HashSet;

trait PnAlgorithm {
    type Input;
    type State: Debug;
    type Msg;

    fn init(ports: usize, i: &Self::Input) -> Self::State;
    fn send(ports: usize, state: &Self::State) -> Vec<Self::Msg>;
    fn receive(state: &mut Self::State, data: &[Self::Msg]);
}

struct PnGraph<A: PnAlgorithm> {
    nodes: Vec<Vec<(usize, usize)>>,
    states: Vec<A::State>
}

impl<A: PnAlgorithm> PnGraph<A> {
    fn new(nodes: Vec<Vec<(usize, usize)>>, input: &[A::Input]) -> PnGraph<A> {
        assert_eq!(nodes.len(), input.len());

        PnGraph {
            states: input.iter().zip(nodes.iter()).map(|(i,n)| A::init(n.len(), i)).collect(),
            nodes: nodes
        }
    }

    fn step(&mut self) {
        let mut in_datas = self.nodes.iter().map(|n| (0..n.len()).map(|x| None).collect::<Vec<_>>())
                                            .collect::<Vec<_>>();

        for (state, node) in self.states.iter().zip(self.nodes.iter()) {
            let out_data = A::send(node.len(), &*state);
            assert_eq!(out_data.len(), node.len());
            for (target, data) in node.iter().zip(out_data.into_iter()) {
                assert!(in_datas[target.0][target.1].is_none());
                in_datas[target.0][target.1] = Some(data);
            }
        }

        for (state, data) in self.states.iter_mut().zip(in_datas.into_iter()) {
            let data = data.into_iter().map(|x| x.unwrap()).collect::<Vec<_>>();
            A::receive(state, &data);
        }
    }
}

struct Bmm;
#[derive(Copy, Clone)]
enum BmmInput {
    White,
    Black
}
#[derive(Debug)]
enum BmmState {
    UrW(usize),
    UrB(usize, HashSet<usize> /* M */, HashSet<usize> /* X */),
    Mr(usize),
    Us,
    Ms(usize)
}
impl BmmState {
    fn is_output(&self) -> bool {
        match *self {
            BmmState::Us | BmmState::Ms(_) => true,
            _ => false
        }
    }

    fn is_matched(&self) -> bool {
        match *self {
            BmmState::Ms(_) => true,
            BmmState::Us => false,
            _ => panic!("algorithm not complete")
        }
    }
}
#[derive(PartialEq, Eq, Debug, Clone)]
enum BmmMsg {
    NoMsg,
    Proposal,
    Matched,
    Accept
}
impl PnAlgorithm for Bmm {
    type Input = BmmInput;
    type State = BmmState;
    type Msg = BmmMsg;

    fn init(ports: usize, i: &BmmInput) -> BmmState {
        match *i {
            BmmInput::White => BmmState::UrW(1),
            BmmInput::Black => BmmState::UrB(1, HashSet::new(), (1..ports+1).collect())
        }
    }

    fn send(ports: usize, state: &BmmState) -> Vec<BmmMsg> {
        use BmmState::*;
        use BmmMsg::*;
        let pr = 1..(ports+1);
        let out = match *state {
            UrW(k) if (k+1)/2 <= ports && k % 2 == 1 =>
                pr.map(|p| if p == (k+1)/2 { Proposal } else { NoMsg }).collect(),
            UrB(k, ref m, ref x) if !m.is_empty() && k % 2 == 0 =>
                pr.map(|p| if p == m.iter().cloned().min().unwrap() { Accept } else { NoMsg }).collect(),
            Mr(i) => pr.map(|_| Matched).collect(),
            _ => pr.map(|_| NoMsg).collect(),
        };
        println!("{:?} -> {:?}", state, out);
        out
    }

    fn receive(state: &mut BmmState, data: &[BmmMsg]) {
        use BmmState::*;
        use BmmMsg::*;

        println!("{:?} <- {:?}", state, data);

        *state = match *state {
            UrW(k) if (k+1)/2 > data.len() && k % 2 == 1 => Us,
            UrW(k) => {
                if k % 2 == 0 && data.iter().any(|m| *m == Accept) {
                    Mr(data.iter().position(|m| *m == Accept).unwrap() + 1)
                } else {
                    UrW(k+1)
                }
            }
            UrB(k, ref m, ref x) if k % 2 == 1 => {
                let mut m = m.clone();
                let mut x = x.clone();
                for (i, msg) in data.iter().enumerate() {
                    if *msg == Proposal {
                        m.insert(i+1);
                    } else if *msg == Matched {
                        x.remove(&(i+1));
                    }
                }
                UrB(k+1, m, x)
            }
            UrB(k, ref m, _) if !m.is_empty() && k % 2 == 0 => Ms(m.iter().cloned().min().unwrap()),
            UrB(k, _, ref x) if x.is_empty() && k % 2 == 0 => Us,
            UrB(k, ref m, ref x) => UrB(k+1, m.clone(), x.clone()),
            Mr(i) => Ms(i),
            Us => Us,
            Ms(i) => Ms(i),
        }
    }
}

struct Vc3;
#[derive(Debug)]
struct Vc3State(BmmState, BmmState);
#[derive(PartialEq, Eq, Debug, Clone)]
struct Vc3Msg(BmmMsg, BmmMsg);
impl PnAlgorithm for Vc3 {
    type Input = ();
    type State = (BmmState, BmmState);
    type Msg = (BmmMsg, BmmMsg);

    fn init(ports: usize, i: &()) -> (BmmState, BmmState) {
        (Bmm::init(ports, &BmmInput::White), Bmm::init(ports, &BmmInput::Black))
    }

    fn send(ports: usize, state: &(BmmState, BmmState)) -> Vec<(BmmMsg, BmmMsg)> {
        Bmm::send(ports, &state.0).into_iter().zip(Bmm::send(ports, &state.1).into_iter())
                                              .map(|(a,b)| (a,b)).collect()
    }

    fn receive(state: &mut (BmmState, BmmState), data: &[(BmmMsg, BmmMsg)]) {
        let (l, r): (Vec<_>, Vec<_>) = data.iter().cloned().unzip();
        Bmm::receive(&mut state.0, &r);
        Bmm::receive(&mut state.1, &l);
    }
}

fn main() {
    let node0 = vec![(1,0), (2,0)];
    let node1 = vec![(0,0), (2,1)];
    let node2 = vec![(0,1), (1,1)];

    let nodes = vec![
        node0,
        node1,
        node2
    ];

//     let nodes = vec![
//         vec![(1,0)],
//         vec![(0,0), (2,0)],
//         vec![(1,1), (3,0)],
//         vec![(2,1), (4,0)],
//         vec![(3,1)]
//     ];

    let nodes = vec![
        vec![(1,1)],
        vec![(2,0), (0,0)],
        vec![(1,0)]
    ];

    let mut graph: PnGraph<Vc3> = PnGraph::new(nodes, &[(),(),()]);

    loop {
        println!("{:#?}", graph.states);
        graph.step();

        let mut dummy = String::new();
        ::std::io::stdin().read_line(&mut dummy).unwrap();
    }
}
