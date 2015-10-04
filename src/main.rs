use std::fmt::Debug;

trait PnAlgorithm {
    type Input;
    type State: Debug;
    type Msg;

    fn init(i: &Self::Input) -> Self::State;
    fn send(ports: usize, state: &Self::State) -> Vec<Self::Msg>;
    fn receive(state: &mut Self::State, data: &[Self::Msg]);
}

struct PnGraph<'a, A: PnAlgorithm> {
    nodes: &'a [&'a [(usize, usize)]],
    states: Vec<A::State>
}

impl<'a, A: PnAlgorithm> PnGraph<'a, A> {
    fn new(nodes: &'a [&'a [(usize, usize)]], input: &[A::Input])
        -> PnGraph<'a, A>
    {
        assert_eq!(nodes.len(), input.len());

        PnGraph {
            nodes: nodes,
            states: input.iter().map(|i| A::init(i)).collect()
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

struct Counter;
impl PnAlgorithm for Counter {
    type Input = bool;
    type State = (i32, usize, bool);
    type Msg = bool;

    fn init(i: &bool) -> (i32, usize, bool) {
        (0, 0, *i)
    }
    fn send(ports: usize, state: &(i32, usize, bool)) -> Vec<bool> {
        (0..ports).map(|port| state.2 && port != state.1).collect()
    }
    fn receive(state: &mut (i32, usize, bool), data: &[bool]) {
        state.2 = false;
        for (port, x) in data.iter().enumerate() {
            if *x {
                state.0 += 1;
                state.1 = port;
                state.2 = true;
                break;
            }
        }
    }
}

fn main() {
    let node_0 = [(3,1), (1,0)];
    let node_1 = [(0,1), (2,0)];
    let node_2 = [(1,1), (3,0)];
    let node_3 = [(2,1), (0,0)];

    // 2-path
    let nodes = &[
        &node_0[..],
        &node_1[..],
        &node_2[..],
        &node_3[..],
    ];

    let mut graph: PnGraph<Counter> = PnGraph::new(nodes, &[true, false, false, false]);

    loop {
        println!("{:?}", graph.states);
        graph.step();

        let mut dummy = String::new();
        ::std::io::stdin().read_line(&mut dummy).unwrap();
    }
}
