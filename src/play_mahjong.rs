use crate::{player_information::PlayerInformation, utils::type_name_of, MahjongAgent, PlayerReactionInformation};
use log::{debug, error, info};
use rand::prelude::*;
use riichi::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GameSummary {
    pub final_points: [GamePoints; 4],
    pub round_histories: Vec<RoundHistory>,
}

/// Start a round of Mahjong with 4 agents.
#[timed::timed(duration(printer = "info!"))]
pub fn play_mahjong(agents: &mut [Box<dyn MahjongAgent>; 4]) -> GameSummary {
    let mut rng = rand::thread_rng();
    let mut engine = Engine::new();
    let mut wall = wall::make_sorted_wall([1; 3]);
    let ruleset = Ruleset::default();
    wall.shuffle(&mut rng);
    let mut rb = RoundBegin {
        ruleset: ruleset.clone(),
        round_id: RoundId { kyoku: 0, honba: 0 },
        wall,
        pot: 0,
        points: [25000; 4],
    };
    engine.begin_round(rb.clone());
    let mut histories = Vec::new();
    loop {
        let mut history = RoundHistory {
            begin: rb.clone(),
            steps: Vec::new(),
            ron: [false; 4],
        };
        while engine.end().is_none() {
            let actor = engine.state().core.actor;
            let info: PlayerInformation = (engine.state(), actor, &rb).into();
            let mut action: Action;
            {
                if let Some(agent) = agents.get_mut(actor.to_u8() as usize) {
                    let agent = agent.as_mut();
                    action = agent.decide(&info);
                    try_register_action(&mut engine, &mut action, actor, agent);
                    debug!("Player {actor} took action {action}");
                } else {
                    unreachable!() // player is out of bounds for some reason, probably engine fault
                }
            }
            for &reactor in &[actor.succ(), actor.oppo(), actor.pred()] {
                // compute reactions
                if let Some(agent) = agents.get_mut(actor.to_u8() as usize) {
                    let agent = agent.as_mut();
                    let info: PlayerInformation = (engine.state(), reactor, &rb).into();
                    let reaction = agent.decide(&PlayerReactionInformation {
                        player_information: info,
                        reactor,
                        action,
                    });
                    if let Some(reaction) = reaction {
                        try_register_reaction(&mut engine, reactor, reaction, actor, agent, action);
                        debug!("Player {reactor} reacted {reaction}");
                    }
                }
            }
            let gs = engine.step();
            history.steps.push(gs);
        } // while round not ended

        // set up new round (if any)
        let round_end = engine.end().as_ref().expect("round not ended yet");
        info!("Round Ended: {round_end:?}");
        // MAYBE fix this to only record it if the result was ron
        history.ron = round_end.agari_result.clone().map(|x| x.is_some());
        histories.push(history);
        if let Some(next_round) = round_end.next_round_id {
            wall = wall::make_sorted_wall([1; 3]);
            wall.shuffle(&mut rng);
            rb = RoundBegin {
                ruleset: ruleset.clone(),
                round_id: next_round,
                wall,
                pot: round_end.pot,
                points: round_end.points,
            };
            engine.begin_round(rb.clone());
        } else {
            break GameSummary {
                final_points: round_end.points,
                round_histories: histories,
            };
        }
    } // loop
}

/// Register the reaction with the engine. If reaction doesn't work, log an error and do nothing.
pub(crate) fn try_register_reaction(
    engine: &mut Engine,
    player: Player,
    reaction: Reaction,
    actor: Player,
    agent: &mut dyn MahjongAgent,
    action: Action,
) {
    let result = engine.register_reaction(player, reaction);
    if let Err(e) = result {
        error!(
            "Agent {} ({}) tried to make an illegal reaction: {} to action: {}. Error: {}.",
            actor,
            type_name_of(agent),
            reaction,
            action,
            e
        );
    }
}

/// Register the action with the engine. If action doesn't work, log an error and use the fallback action.
pub(crate) fn try_register_action(
    engine: &mut Engine,
    action: &mut Action,
    actor: Player,
    agent: &mut dyn MahjongAgent,
) {
    let result = &engine.register_action(*action);
    if let Err(e) = result {
        error!(
            "Agent {} ({}) tried to make an illegal action: {}. Action returned {}. Using fallback action.",
            actor,
            type_name_of(agent),
            action,
            e
        );
        let state = engine.state();
        *action = fallback_action(state.core.draw, &state.closed_hands[actor.to_u8() as usize]);
        let result = &engine.register_action(*action);
        assert!(result.is_ok(), "Fallback action did not work");
    }
}

/// Default action if agent tries to do anything illegal.
/// Either tsumogiri (if drew this turn) or discard the first tile in hand (if didn't draw this turn, such as if you melded)
pub(crate) fn fallback_action(draw: Option<Tile>, hand: &TileSet37) -> Action {
    match draw {
        Some(t) => Action::Discard(Discard {
            tile: t,
            called_by: Player::new(0),
            declares_riichi: false,
            is_tsumogiri: true,
        }),
        None => Action::Discard(Discard {
            tile: hand
                .iter_tiles()
                .next()
                .expect("Did not have any tiles to discard"),
            called_by: Player::new(0),
            declares_riichi: false,
            is_tsumogiri: false,
        }),
    }
}
