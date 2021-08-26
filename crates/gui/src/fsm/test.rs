use super::*;

#[derive(Debug)]
struct StateA(usize);

#[derive(Debug)]
struct StateB(usize);

impl State for StateA {
    type Data = ();
    fn update(
        &mut self,
        _data: &mut Self::Data,
        _cell: &StateCell<Self::Data>,
    ) -> StateReturn<Self::Data> {
        StateReturn::NextFrame(vec![StateCommand::Push(TypeId::of::<StateB>())])
    }
}

impl State for StateB {
    type Data = ();
    fn update(
        &mut self,
        _data: &mut Self::Data,
        cell: &StateCell<Self::Data>,
    ) -> StateReturn<Self::Data> {
        let a = cell.get_mut::<StateA>().unwrap();
        a.0 += 1;
        StateReturn::NextFrame(vec![])
    }
}

// cargo +nightly miri test
#[test]
fn state_cell_soundness() {
    let mut fsm = Fsm::default();
    fsm.insert(StateA(0));
    fsm.insert(StateB(0));

    let data = &mut ();
    fsm.push::<StateA>(data);

    for i in 0..=3 {
        fsm.update(data);
    }

    assert_eq!(fsm.get::<StateA>().unwrap().0, 3);
}
