use riichi::prelude::*;
use crate::play_mahjong;
use crate::strategies::Strategy;

/// Represents the information that a player can see.
/// See [`riichi::model::State`] and [`riichi::model::StateCore`] for more information on these fields.
#[derive(Clone)]
pub struct PlayerInformation {
    // attributes from RoundBegin
    pub round_id: RoundId,
    pub pot: GamePoints,
    pub points: [GamePoints; 4],
    // attributes from StateCore
    pub seq: u8,
    pub actor: Player,
    pub num_drawn_head: u8,
    pub num_drawn_tail: u8,
    // it's a bit irritating that we have to pull this info from the wall but what else can we do honestly
    pub dora_indicators: Vec<Tile>,
    // can only view own draw
    pub draw: Option<Tile>,
    pub incoming_meld: Option<Meld>,
    // can only view furiten flags for self
    pub furiten: FuritenFlags,
    pub riichi: [Option<Riichi>; 4],
    // attributes from State, excluding StateCore
    pub melds: [Vec<Meld>; 4],
    pub closed_hand: TileSet37,
    pub discards: [Vec<Discard>; 4],
    pub discard_sets: [TileMask34; 4],
}

impl From<(&State, Player, &RoundBegin)> for PlayerInformation {
    fn from((state, player, round_begin): (&State, Player, &RoundBegin)) -> Self {
        let wall = &round_begin.wall;
        // if you are the current player, then select draw, else
        let draw = (player == state.core.actor)
            .then_some(state.core.draw)
            .flatten();
        // get dora indicators
        let dora_indicators: Vec<_> = (0..state.core.num_dora_indicators as usize)
            .map(|x| wall::dora_indicator(wall, x))
            .collect();
        PlayerInformation {
            round_id: round_begin.round_id,
            pot: round_begin.pot,
            points: round_begin.points,
            seq: state.core.seq,
            actor: state.core.actor,
            num_drawn_head: state.core.num_drawn_head,
            num_drawn_tail: state.core.num_drawn_tail,
            dora_indicators,
            draw,
            incoming_meld: state.core.incoming_meld,
            furiten: state.core.furiten[player.to_usize()],
            riichi: state.core.riichi,
            melds: state.melds.clone(),
            closed_hand: state.closed_hands[player.to_usize()].clone(),
            discards: state.discards.clone(),
            discard_sets: state.discard_sets,
        }
    }
}

// define helpful trait aliases for stuff
pub trait ActionStrategy: Strategy<PlayerInformation, Action> {}
impl<T: Strategy<PlayerInformation, Action>> ActionStrategy for T {}
pub trait ReactionStrategy: Strategy<(PlayerInformation, Action, Player), Option<Reaction>> {}
impl<T: Strategy<(PlayerInformation, Action, Player), Option<Reaction>>> ReactionStrategy for T {}
#[allow(clippy::module_name_repetitions)] // having "Mahjong" as a trait name is probably weird
pub trait MahjongAgent: ActionStrategy + ReactionStrategy {}
impl<T: ActionStrategy + ReactionStrategy> MahjongAgent for T {}

/// Really stupid mahjong player, doesn't react and always does the fallback action (i.e. tsumogiri)
pub struct VeryStupid;
impl Strategy<PlayerInformation, Action> for VeryStupid {
    fn decide(&mut self, player_info: &PlayerInformation) -> Action {
        play_mahjong::fallback_action(player_info.draw, &player_info.closed_hand)
    }
}
impl Strategy<(PlayerInformation, Action, Player), Option<Reaction>> for VeryStupid {
    fn decide(&mut self, _state: &(PlayerInformation, Action, Player)) -> Option<Reaction> {
        None
    }
}
