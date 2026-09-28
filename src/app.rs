mod game {
    use core::fmt::{Display, Formatter, Result};
    use rand::prelude::*;

    #[derive(Clone, Debug)]
    enum DiceState {
        Point,
        Reroll,
        Fail,
    }

    impl Display for DiceState {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Self::Point => write!(f, "🧠"),
                Self::Reroll => write!(f, "👣"),
                Self::Fail => write!(f, "💥"),
            }
        }
    }

    #[derive(Clone)]
    enum DiceColor {
        Red,
        Yellow,
        Green,
    }

    #[derive(Clone)]
    struct Dice {
        color: DiceColor,
        state: DiceState,
    }

    impl Dice {
        fn roll(&mut self) {
            let dice_faces = match self.color {
                DiceColor::Green => [
                    (DiceState::Point, 3),
                    (DiceState::Reroll, 2),
                    (DiceState::Fail, 1),
                ],
                DiceColor::Yellow => [
                    (DiceState::Point, 2),
                    (DiceState::Reroll, 2),
                    (DiceState::Fail, 2),
                ],
                DiceColor::Red => [
                    (DiceState::Point, 1),
                    (DiceState::Reroll, 2),
                    (DiceState::Fail, 3),
                ],
            };

            let mut rng = rand::rng();
            let outcome = dice_faces
                .choose_weighted(&mut rng, |item| item.1)
                .unwrap()
                .0
                .clone();
            self.state = outcome;
        }

        fn new(color: DiceColor) -> Self {
            Self {
                color,
                state: DiceState::Reroll,
            }
        }
    }

    impl Display for Dice {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self.color {
                DiceColor::Red => write!(f, "🎲🟥[{}]", self.state),
                DiceColor::Yellow => write!(f, "🎲🟨[{}]", self.state),
                DiceColor::Green => write!(f, "🎲🟩[{}]", self.state),
            }
        }
    }

    struct Player {
        name: String,
        points: i32,
    }

    impl Player {
        fn new(name: String) -> Self {
            Self { name, points: 0 }
        }
    }

    enum Action {
        Roll,
        Stay,
    }

    struct Turn<'a> {
        dicecup: Vec<Dice>,
        player: &'a mut Player,
    }

    impl<'a> Turn<'a> {
        fn new(player: &'a mut Player) -> Self {
            let dicecup: Vec<Dice> = vec![
                Dice::new(DiceColor::Red),
                Dice::new(DiceColor::Red),
                Dice::new(DiceColor::Red),
                Dice::new(DiceColor::Yellow),
                Dice::new(DiceColor::Yellow),
                Dice::new(DiceColor::Yellow),
                Dice::new(DiceColor::Yellow),
                Dice::new(DiceColor::Green),
                Dice::new(DiceColor::Green),
                Dice::new(DiceColor::Green),
                Dice::new(DiceColor::Green),
                Dice::new(DiceColor::Green),
                Dice::new(DiceColor::Green),
            ];

            Self { dicecup, player }
        }

        fn play_turn(&mut self) {
            let mut gathered_points = 0;

            let mut hand: Vec<Dice> = Vec::new();
            let mut table: Vec<Dice> = Vec::new();
            let mut rerolls: Vec<Dice> = Vec::new();
            let mut fails: Vec<Dice> = Vec::new();

            loop {
                let action = Self::get_action();
                match action {
                    Action::Roll => {
                        self.dicecup.shuffle(&mut rand::rng());

                        while hand.len() < 3 {
                            hand.push(self.dicecup.pop().expect("Dicecup shouldn't be run out"));
                        }

                        while let Some(mut dice) = hand.pop() {
                            dice.roll();
                            println!("{dice}");
                            match dice.state {
                                DiceState::Fail => {
                                    fails.push(dice);
                                }
                                DiceState::Point => {
                                    gathered_points += 1;
                                    table.push(dice);
                                }
                                DiceState::Reroll => {
                                    rerolls.push(dice);
                                }
                            }
                        }

                        if fails.len() >= 3 {
                            println!(
                                "Aww shucks! {}'s turn ended with a grand explosion, taking a shotgun to the face {} times",
                                self.player.name, fails.len()
                            );
                            println!(
                                "By the way, {} has {} points and missed out on {} points",
                                self.player.name, self.player.points, gathered_points
                            );
                            return;
                        }

                        // Refill hand from rolled rerolls
                        while let Some(dice) = rerolls.pop() {
                            hand.push(dice);
                        }

                        // Refill rest of hand from dicecup
                        while hand.len() < 3 {
                            if let Some(dice) = self.dicecup.pop() {
                                hand.push(dice);
                            }
                            // Refill dicecup from table if necessary
                            else {
                                println!(
                                    "Woah what a turn! We need to refill the dice cup from the table!"
                                );
                                while let Some(dice) = table.pop() {
                                    self.dicecup.push(dice);
                                }
                            }
                        }
                    }
                    Action::Stay => {
                        self.player.points += gathered_points;
                        println!(
                            "Turn of {} ended with {} points gained, now at {}",
                            self.player.name, gathered_points, self.player.points
                        );
                        return;
                    }
                };
            }
        }
    }
}

/// Application.
#[derive(Debug, Default)]
pub struct App {
    /// should the application exit?
    pub should_quit: bool,
    /// counter
    pub counter: u8,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set should_quit to true to quit the application.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_app_increment_counter() {
        let mut app = App::default();
        app.increment_counter();
        assert_eq!(app.counter, 1);
    }

    #[test]
    fn test_app_decrement_counter() {
        let mut app = App::default();
        app.decrement_counter();
        assert_eq!(app.counter, 0);
    }
}
