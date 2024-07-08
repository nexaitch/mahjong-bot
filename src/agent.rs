use riichi::model::{Action, Discard, Reaction};
use riichi::prelude::Player;

use crate::play_mahjong;
use crate::player_information::PlayerInformation;
use crate::strategies::Strategy;

/// Information required to decide how a player should react
pub struct PlayerReactionInformation {
    pub player_information: PlayerInformation,
    pub reactor: Player,
    pub action: Action,
}
// define helpful trait aliases for stuff
pub trait ActionStrategy: Strategy<PlayerInformation, Action> {}
impl<T: Strategy<PlayerInformation, Action>> ActionStrategy for T {}
pub trait ReactionStrategy:
    Strategy<PlayerReactionInformation, Option<Reaction>>
{
}
impl<T: Strategy<PlayerReactionInformation, Option<Reaction>>> ReactionStrategy for T {}
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
impl Strategy<PlayerReactionInformation, Option<Reaction>> for VeryStupid {
    fn decide(&mut self, _state: &PlayerReactionInformation) -> Option<Reaction> {
        None
    }
}

pub struct Modular<R, D, K, T, R_, C> {
    /// `Strategy<PlayerInformation, bool>` to determine if a player should call riichi.
    riichi: R,
    /// `Strategy<PlayerInformation, Discard>` to determine how a player should discard.
    discard: D,
    /// `Strategy<PlayerInformation, bool>` to determine if a player should ankan/kakan.
    kan: K,
    /// `Strategy<PlayerInformation, bool>` to determine if a player should tsumo.
    tsumo: T,
    // reactions
    /// `Strategy<(PlayerInformation, Action, Player), bool>` to determine if a player should ron
    ron: R_,
    /// `Strategy<(PlayerInformation, Action, Player), Option<Reaction>>` to determine how a player should call.
    /// Unlike the other functions here, this is called even if there are no possible melds to make.
    call: C,
}

pub fn modular<R, D, K, T, R_, C>(
    riichi: R,
    discard: D,
    kan: K,
    tsumo: T,
    ron: R_,
    call: C,
) -> Modular<R, D, K, T, R_, C>
where
    R: Strategy<PlayerInformation, bool>,
    D: Strategy<PlayerInformation, Discard>,
    K: Strategy<PlayerInformation, bool>,
    T: Strategy<PlayerInformation, bool>,
    R_: Strategy<PlayerReactionInformation, bool>,
    C: Strategy<PlayerReactionInformation, Option<Reaction>>,
{
    Modular {
        riichi,
        discard,
        kan,
        tsumo,
        ron,
        call,
    }
}

impl<R, D, K, T, R_, C> Strategy<PlayerInformation, Action> for Modular<R, D, K, T, R_, C>
where
R: Strategy<PlayerInformation, bool>,
D: Strategy<PlayerInformation, Discard>,
K: Strategy<PlayerInformation, bool>,
T: Strategy<PlayerInformation, bool>,
R_: Strategy<PlayerReactionInformation, bool>,
C: Strategy<PlayerReactionInformation, Option<Reaction>>,
{
    #[allow(dead_code, unused_variables, unreachable_code, clippy::diverging_sub_expression)]
    fn decide(&mut self, state: &PlayerInformation) -> Action {
        if let Some(draw) = state.draw {
            if todo!("can tsumo") && self.tsumo.decide(state) {
                return Action::TsumoAgari(draw);
            }
            if state.riichi[state.actor.to_usize()].is_some() {
                // tsumogiri
                return Action::Discard(Discard {
                    tile: draw,
                    called_by: state.actor,
                    declares_riichi: false,
                    is_tsumogiri: true,
                });
            }
            if todo!("can an/kakan") && self.kan.decide(state) {
                // if tile is in hand, it's ankan, else kakan
                return if state.closed_hand[draw.normal_encoding() as usize] > 0 {
                    Action::Ankan(draw)
                } else {
                    Action::Kakan(draw)
                };
            }
        }
        let r = todo!("can riichi") && self.riichi.decide(state);
        let mut d = self.discard.decide(state);
        d.declares_riichi = r;
        Action::Discard(d)
    }
}

impl<R, D, K, T, R_, C> Strategy<PlayerReactionInformation, Option<Reaction>>
    for Modular<R, D, K, T, R_, C>
where
R: Strategy<PlayerInformation, bool>,
D: Strategy<PlayerInformation, Discard>,
K: Strategy<PlayerInformation, bool>,
T: Strategy<PlayerInformation, bool>,
R_: Strategy<PlayerReactionInformation, bool>,
C: Strategy<PlayerReactionInformation, Option<Reaction>>,
{
    #[allow(dead_code, unused_variables, unreachable_code, clippy::diverging_sub_expression)]
    fn decide(&mut self, pri: &PlayerReactionInformation) -> Option<Reaction> {
        let PlayerReactionInformation {player_information, reactor, action} = pri;
        let ron = todo!("can ron") && self.ron.decide(pri);
        ron.then_some(Reaction::RonAgari)
            .or_else(|| self.call.decide(pri))
    }
}
