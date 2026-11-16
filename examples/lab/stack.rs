//! Lab · stack: the card stack (`widgets::card_stack`) in the lab's look: hover or press a card and
//! the ones beneath fan out, card n turning angle0 × falloff^n after delay × n.
use lab::el::{El, Paint};
use lab::experiment::{Experiment, Params, frame, spec};
use lab::widgets::{Edit, Spec, Stack as Deck, StackMsg, card_stack};

static DEFS: [(&str, Spec); 3] = [("angle0", spec(0.0, 90.0, 1.0, 0, "°")), ("falloff", spec(0.1, 1.0, 0.01, 2, "")), ("delay", spec(0.0, 200.0, 1.0, 0, "ms"))];

#[derive(Clone, Debug, PartialEq)]
pub struct Stack { pub p: Params, pub deck: Deck }
impl Default for Stack {
    fn default() -> Self { let d = Deck::default(); Self { p: Params::new(&DEFS, &[d.angle0, d.falloff, d.delay_ms]), deck: d } }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Deck(StackMsg), Num(usize, Edit) }

impl Experiment for Stack {
    type Msg = Msg;
    fn view(&self, key: &str) -> El<Msg> {
        // Cards in the lab's look: near the ground, drawn by their hairline outlines; the one under the
        // pointer takes the accent.
        let mut d = self.deck.clone();
        d.paints = (0..d.count).map(|i| if d.under() == Some(i) { Paint::ACCENT.alpha(0.55) } else { Paint::tone(0.07) }).collect();
        d.outline = 1.2;
        d.card = (0.95, 0.6);
        let piece = card_stack(&format!("{key}/deck"), &d, 300.0, 170.0, Msg::Deck);
        let formula = format!("angle(n) = {:.0}° × {:.2}^n,  delay(n) = {:.0} ms × n", d.angle0, d.falloff, d.delay_ms);
        frame(key, "stack", "hover to fan", piece, formula, &self.p, Msg::Num, "springs per card · staggered")
    }
    fn update(&mut self, m: Msg, now: f32) {
        match m {
            Msg::Deck(m) => self.deck.update(m, now),
            Msg::Num(i, e) => { self.p.edit(i, e); (self.deck.angle0, self.deck.falloff, self.deck.delay_ms) = (self.p.get(0), self.p.get(1), self.p.get(2)); }
        }
    }
    fn step(&mut self, now: f32) -> bool { self.deck.step(now) }
    fn moving(&self) -> bool { self.deck.moving() }
}
