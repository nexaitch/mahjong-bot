use itertools::Itertools;
pub trait TileSetExt {
    fn to_shorthand(&self) -> String;
}

impl TileSetExt for riichi::prelude::TileSet37 {
    /// Converts a hand in the [`riichi::prelude::TileSet37`] format to human-readable shorthand (e.g. 123m456p789s1155z)
    fn to_shorthand(&self) -> String {
        self.iter_tiles()
            .sorted_by_key(|x| x.suit_char())
            .chunk_by(|x| x.suit_char()) // group by suit
            .into_iter()
            .map(|(suit_char, tiles)| {
                format!(
                    "{}{}",
                    tiles.map(riichi::prelude::Tile::num).join(""),
                    suit_char
                )
            }) // convert to string representation of tiles in one suit
            .filter(|x| x.len() > 1) // ignore zero-tile suits
            .join("")
    }
}

/// Gets the type name of an object. Used for debugging.
#[must_use]
pub(crate) fn type_name_of<T>(_: &T) -> String
where
    T: ?Sized,
{
    std::any::type_name::<T>().to_string()
}

#[cfg(test)]
mod tests {

    use super::*;
    use test_case::test_case;

    #[test_case("123m456p789s11155z"; "basic test case")]
    #[test_case("1112345678999m"; "empty suits")]
    fn test_shorthand_tile37(s: &str) {
        let tiles = riichi::prelude::tiles_from_str(s);
        let tiles = tiles.collect::<riichi::prelude::TileSet37>();
        assert_eq!(tiles.to_shorthand(), s);
    }
}
