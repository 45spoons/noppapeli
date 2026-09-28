use core::fmt;
use std::io;
use rand::prelude::*;

use color_eyre::Result;
use ratatui::{
    crossterm::event,
    layout::{Constraint, Layout, Spacing},
    symbols::merge::MergeStrategy,
    style::{Style},
    widgets::*,
    text::{Line},
    DefaultTerminal, Frame,
};
use tachyonfx::{fx, EffectManager, pattern};

/// Application.
pub mod app;

/// Terminal events handler.
pub mod event;

/// Widget renderer.
pub mod ui;

/// Terminal user interface.
pub mod tui;

/// Application updater.
pub mod update;


#[derive(Clone, Debug)]
enum DiceState {
    Point,
    Reroll,
    Fail,
}

impl fmt::Display for DiceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            DiceColor::Green => {[(DiceState::Point, 3), (DiceState::Reroll, 2), (DiceState::Fail, 1)]},
            DiceColor::Yellow => {[(DiceState::Point, 2), (DiceState::Reroll, 2), (DiceState::Fail, 2)]},
            DiceColor::Red => {[(DiceState::Point, 1), (DiceState::Reroll, 2), (DiceState::Fail, 3)]},
        };

        let mut rng = rand::rng();
        let outcome = dice_faces.choose_weighted(&mut rng, |item| item.1).unwrap().0.clone();
        self.state = outcome;
    }

    fn new(color: DiceColor) -> Self {
        Self {
            color,
            state: DiceState::Reroll,
        }
    }
}

impl fmt::Display for Dice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        Self {
            name,
            points: 0,
        }
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

        Self {
            dicecup,
            player,
        }
    }

    fn _readiness_check(&self) {
        println!("Give {} the control, are you ready? (press ENTER)", self.player.name);
        io::stdin()
            .read_line(&mut String::new())
            .expect("Failed to read line");
    }

    fn play_turn(&mut self) {
        let mut gathered_points = 0;
        let mut fails = 0;

        self._readiness_check();
        println!("NEW TURN: {} [ {} points ] GO! (enter \"r\" to roll and \"s\" to redeem your points)", self.player.name, self.player.points);

        let mut hand: Vec<Dice> = Vec::new();
        let mut table: Vec<Dice> = Vec::new();
        let mut rerolls: Vec<Dice> = Vec::new();

        loop {
            let action = Self::_get_action();
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
                                fails += 1;
                                table.push(dice);
                            },
                            DiceState::Point => {
                                gathered_points += 1;
                                table.push(dice);
                            },
                            DiceState::Reroll => {
                                rerolls.push(dice);
                            },
                        }
                    }

                    if fails >= 3 {
                        println!("Aww shucks! {}'s turn ended with a grand explosion, taking a shotgun to the face {} times", self.player.name, fails);
                        println!("By the way, {} has {} points and missed out on {} points", self.player.name, self.player.points, gathered_points);
                        return
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
                            println!("Woah what a turn! We need to refill the dice cup from the table!");
                                while let Some(dice) = table.pop() {
                                self.dicecup.push(dice);
                            }
                        }
                    }
                },
                Action::Stay => {
                    self.player.points += gathered_points;
                    println!("Turn of {} ended with {} points gained, now at {}", self.player.name, gathered_points, self.player.points);
                    return
                },
            };
        }
    }

    fn _get_action() -> Action {
        loop {
            let mut action = String::new();
            io::stdin()
                .read_line(&mut action)
                .expect("Failed to read line");

            let action = match action.trim() {
                "r" => Some(Action::Roll),
                "s" => Some(Action::Stay),
                _ => None,
            };

            if let Some(action) = action {
                return action
            }
            println!("Type \"r\" to ROLL and \"s\" to STAY");
            println!("Staying will end your turn and save your points, roll if you wish to push your luck");
        }
    }
}

fn gather_players() -> Vec<Player> {
    let mut players: Vec<Player> = Vec::new();

    loop {
        let mut name = String::new();

        io::stdin()
            .read_line(&mut name)
            .expect("Failed to read line");
        name = name.trim().to_string();

        if name.is_empty() {
            println!("Done adding players I see");
            break players
        }

        let new_player = Player::new(name);
        println!("Added {}", new_player.name);
        players.push(new_player);
    }
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut effects: EffectManager<()> = EffectManager::default();

    loop {
        terminal.draw(|frame| render(frame))?;
        if event::read()?.is_key_press() {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    let [title_bar_area, main_area] = Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)])
        .areas(frame.area());

    let [gamba_area, side_bar_area] = Layout::horizontal([Constraint::Percentage(75), Constraint::Fill(1)])
        .spacing(Spacing::Overlap(1))
        .areas(main_area);

    let title_bar_widget = Block::default()
        .title(Line::from(" NOPPAPELI ").centered())
        .borders(Borders::TOP)
        .border_style(Style::default()
        .fg(ratatui::style::Color::LightGreen));

    let gamba_widget = Block::default()
        .title(" GAMBA!! ")
        .merge_borders(MergeStrategy::Fuzzy)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let side_bar_widget = Block::default()
        .title(" Side items ")
        .merge_borders(MergeStrategy::Fuzzy)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let fg_shift = [1440.0, 0.0, 0.0];
    let timer = 2000;

    let hsl_title_xform = fx::hsl_shift_fg(fg_shift, timer)
        .with_pattern(pattern::SweepPattern::left_to_right(160))
        .with_area(title_bar_area);

    let title_effect = fx::repeating(fx::remap_alpha(0.3333, 0.6667, hsl_title_xform));

    frame.render_widget(title_bar_widget, title_bar_area);
    frame.render_widget(gamba_widget, gamba_area);
    frame.render_widget(side_bar_widget, side_bar_area);
}

fn main() -> Result<> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut effects: EffectManager<()> = EffectManager::default();
    let _ = run(terminal);

    // let mut players = gather_players();

    // while players.is_empty() {
    //     println!("Hey, you didn't add any players..?");
    //     players = gather_players();
    // }

    // 'game:loop {
    //     for player in &mut players {
    //         let mut turn = Turn::new(player);
    //         turn.play_turn();
    //         if player.points >= 20 {
    //             println!("{} WINS! CONCRAPULATIONS!! 💩 :DD", player.name);
    //             break 'game;
    //         }
    //     }
    // };

    ratatui::restore();
    Ok(())
}
