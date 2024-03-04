use gloo::timers::callback::Interval;
use yew::prelude::*;

use crate::apps::tiles::{Blocks, Interpreter, Neighborhood, Rgba, GRID_SIZE};
use crate::components::*;

#[derive(Properties, PartialEq, Debug)]
pub struct SimulationProps {
    #[prop_or_default]
    pub class: Classes,

    pub seed: u64,
    pub update: Vec<u8>,
}

pub struct Simulation {
    prev: Blocks,
    next: Blocks,

    interpreter: Interpreter,

    _interval: Interval,
}

// TODO: Show error state if broken

impl Component for Simulation {
    type Message = ();

    type Properties = SimulationProps;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let link = ctx.link().clone();

        Self {
            prev: Blocks::default(),
            next: Blocks::from_seed(props.seed),

            interpreter: Interpreter::new(&props.update).unwrap(),

            _interval: Interval::new(1000, move || {
                link.send_message(());
            }),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        html! {
            <Grid prev={self.prev} next={self.next} class={props.class.clone()} />
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        // Prevent accidental changes to the Message type.
        let () = msg;

        let prev = self.next;
        self.prev = self.next;

        // These functions do grid coordinate math with edge wrapping that avoids overflowing
        // usize. There's probably a smarter way to do this.

        fn sub(a: usize, b: usize) -> usize {
            (a + GRID_SIZE - b) % GRID_SIZE
        }

        fn add(a: usize, b: usize) -> usize {
            (a + b) % GRID_SIZE
        }

        for r in 0..GRID_SIZE {
            for c in 0..GRID_SIZE {
                let neighborhood: Neighborhood = (
                    prev[(sub(r, 1), sub(c, 1))].into(),
                    prev[(sub(r, 1), c)].into(),
                    prev[(sub(r, 1), add(c, 1))].into(),
                    prev[(r, sub(c, 1))].into(),
                    prev[(r, c)].into(),
                    prev[(r, add(c, 1))].into(),
                    prev[(add(r, 1), sub(c, 1))].into(),
                    prev[(add(r, 1), c)].into(),
                    prev[(add(r, 1), add(c, 1))].into(),
                );

                self.next[(r, c)] = Rgba::from(self.interpreter.eval(neighborhood).unwrap());
            }
        }

        true
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        // If the update module changes, we have to rebuild the whole wasmi instance.
        let new_update = &ctx.props().update;
        if new_update != &old_props.update {
            self.interpreter = Interpreter::new(new_update).unwrap();
        }

        // If the seed changes, restart from its initial state.
        let new_seed = ctx.props().seed;
        if new_seed != old_props.seed {
            self.prev = Blocks::default();
            self.next = Blocks::from_seed(new_seed);
        }

        true
    }
}
