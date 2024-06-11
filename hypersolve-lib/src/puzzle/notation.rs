use itertools::Itertools;
use num_enum::TryFromPrimitive;

use super::*;

/// Notation types for twists
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, strum::Display, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum Notation {
    /// Standard notation as described [here](https://hypercubing.xyz/notation/)
    #[default]
    Standard,
    /// Notation used by [MC4D](https://superliminal.com/cube/)
    MC4D,
}

/// Errors for parsing standard twist notation
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseStandardTwistError {
    #[error("invalid twist slice mask `{0}`")]
    InvalidSliceMask(String),
    #[error("missing twist axis")]
    MissingAxis,
    #[error("invalid twist axis `{0}`")]
    InvalidFace(String),
    #[error("missing twist direction")]
    MissingDirection,
    #[error("invalid twist piece `{0}`")]
    InvalidPiece(String),
    #[error("twist `{0}` does nothing")]
    TwistDoesNothing(String),
    #[error("unknown twist modifier `{0}`")]
    UnknownModifier(String),
    #[error("unexpected `{0}`")]
    UnexpectedValue(String),
}

/// Errors for parsing MC4D twist notation
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseMC4DTwistError {
    #[error("missing twist ID")]
    MissingId,
    #[error("invalid twist ID `{0}`")]
    InvalidId(String),
    #[error("missing twist amount")]
    MissingAmount,
    #[error("invalid twist amount `{0}`")]
    InvalidAmount(String),
    #[error("missing twist slice mask")]
    MissingSliceMask,
    #[error("invalid twist slice mask `{0}`")]
    InvalidSliceMask(String),
    #[error("unexpected value `{0}`")]
    UnexpectedValue(String),
}

/// Errors for parsing twist notation
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[allow(clippy::upper_case_acronyms)]
pub enum ParseTwistError {
    #[error("standard twist notation error: {0}")]
    Standard(ParseStandardTwistError),
    #[error("MC4D twist notation error: {0}")]
    MC4D(ParseMC4DTwistError),
    #[error("unrecognized twist notation `{0}`")]
    UnrecognizedNotation(String),
}

impl From<ParseStandardTwistError> for ParseTwistError {
    fn from(value: ParseStandardTwistError) -> Self {
        ParseTwistError::Standard(value)
    }
}
impl From<ParseMC4DTwistError> for ParseTwistError {
    fn from(value: ParseMC4DTwistError) -> Self {
        ParseTwistError::MC4D(value)
    }
}

impl Notation {
    /// Parses a twist from its notation
    pub fn parse_twist(&self, notation: &str) -> Result<Twist, ParseTwistError> {
        match self {
            Notation::Standard => Ok(parse_standard_twist_string(notation)?),
            Notation::MC4D => Ok(parse_mc4d_twist_string(notation)?),
        }
    }

    /// Parses a twist sequence from its notation
    pub fn parse_twist_sequence(&self, notation: &str) -> Result<TwistSequence, ParseTwistError> {
        let twists = notation.split_whitespace();

        let mut result = Vec::new();

        for twist_string in twists {
            result.push(self.parse_twist(twist_string)?)
        }

        Ok(TwistSequence(result))
    }

    /// Formats the twist according to this notation
    pub fn format_twist(&self, twist: &Twist) -> String {
        match self {
            Notation::Standard => twist_to_standard_string(twist),
            Notation::MC4D => twist_to_mc4d_string(twist),
        }
    }

    /// Formats the twist sequence according to this notation
    pub fn format_twist_sequence(&self, twist_seq: &TwistSequence) -> String {
        twist_seq.iter().map(|t| self.format_twist(t)).join(" ")
    }
}

/// Parses the layer from a standard string
fn layer_from_standard_string(s: &str) -> Result<Layer, ParseStandardTwistError> {
    match s.trim_start_matches("{").trim_end_matches("}") {
        "1" => Ok(Layer::This),
        "2" => Ok(Layer::Other),
        "1-2" | "1,2" => Ok(Layer::Both),
        _ => Err(ParseStandardTwistError::InvalidSliceMask(s.into())),
    }
}

fn layer_to_standard_string(layer: Layer) -> &'static str {
    match layer {
        Layer::This => "",
        Layer::Other => "{2}",
        Layer::Both => "{1-2}",
    }
}

/// Returns the standard string for this twist    
fn twist_to_standard_string(twist: &Twist) -> String {
    if twist.layer == Layer::Other {
        return twist_to_standard_string(&Twist::new(
            twist.face.opposite(),
            twist.direction.reverse(),
            Layer::This,
        ));
    }

    let neighboring_faces = twist
        .direction
        .signs_within_face()
        .into_iter()
        .zip(twist.face.basis_faces())
        .filter_map(|(s, f)| match s {
            ZeroOrSign::Zero => None,
            ZeroOrSign::Pos => Some(f * Sign::Pos),
            ZeroOrSign::Neg => Some(f * Sign::Neg),
        })
        .sorted_by_key(|f| match f.axis() {
            Axis::Y => 0,
            Axis::Z => 1,
            Axis::X => 2,
            Axis::W => 3,
        });

    let faces = std::iter::once(twist.face).chain(neighboring_faces);

    let mut string = layer_to_standard_string(twist.layer).to_owned();

    string.extend(faces.map(|f| f.to_string()));

    if twist.direction.is_double() {
        string.push('2')
    }

    string
}

/// Returns the MC4D string for this twist
fn twist_to_mc4d_string(twist: &Twist) -> String {
    use once_cell::sync::Lazy;
    use std::collections::HashMap;

    let mut twist = twist.clone();

    static MC4D_TWIST_IDS: Lazy<HashMap<(Face, TwistDirection), usize>> = Lazy::new(|| {
        mc4d_twist_order()
            .enumerate()
            .filter_map(|(i, twist)| Some((twist?, i)))
            .collect()
    });

    let dir: TwistDirection = twist.direction;
    if let Some(quarter_turn) = dir.half() {
        twist.direction = quarter_turn;
        return format!("{0} {0}", twist_to_mc4d_string(&twist));
    }
    let sticker_id = *MC4D_TWIST_IDS.get(&(twist.face, twist.direction)).unwrap();
    let direction_id = 1;
    let layer_mask = twist.layer as u8;
    format!("{sticker_id},{direction_id},{layer_mask}")
}

/// Creates a twist from its standard notation
fn parse_standard_twist_string(mut s: &str) -> Result<Twist, ParseStandardTwistError> {
    // Get the twist layer
    let layer = if let Some(pos) = s.find('}') {
        let (layer_str, remaining) = s.split_at(pos + 1);
        s = remaining;

        layer_from_standard_string(layer_str)?
    } else {
        Layer::This
    };

    let mut chars = s.chars();

    let face_str = chars.next().ok_or(ParseStandardTwistError::MissingAxis)?;

    // get the twist face
    let face = Face::from_symbol_upper_str(face_str.to_string().as_str())
        .ok_or(ParseStandardTwistError::InvalidFace(face_str.into()))?;

    // get the twist piece faces
    let mut dir_faces = Vec::new();
    for c in chars.peeking_take_while(|c| c.is_alphabetic()) {
        dir_faces.push(
            Face::from_symbol_upper_str(c.to_string().as_str())
                .ok_or(ParseStandardTwistError::InvalidFace(c.to_string()))?,
        );
    }

    if dir_faces.len() == 0 {
        return Err(ParseStandardTwistError::MissingDirection);
    }

    // ensure no repeated axes
    if !dir_faces
        .iter()
        .chain(std::iter::once(&face))
        .map(|f| f.axis())
        .all_unique()
    {
        let piece = dir_faces
            .iter()
            .chain(std::iter::once(&face))
            .map(|f| f.to_string())
            .join("");

        return Err(ParseStandardTwistError::InvalidPiece(piece));
    }

    // get the direction
    let face_vec: Vector4<i32> = dir_faces.into_iter().map(|f| Vector4::<i32>::from(f)).sum();
    let mut direction = TwistDirection::from_signs_within_face(Vector3::from_function(|i| {
        face_vec.dot(face.basis()[i])
    }))
    .unwrap();

    // modifiers
    let mut double = false;
    let mut inverse = false;
    if let Some(modifier) = chars.next() {
        if modifier == '2' {
            double = true;

            if let Some(modifier) = chars.next() {
                if modifier == '\'' {
                    inverse = true;
                } else {
                    return Err(ParseStandardTwistError::UnknownModifier(
                        modifier.to_string(),
                    ));
                }
            }
        } else if modifier == '\'' {
            inverse = true;
        } else {
            return Err(ParseStandardTwistError::UnknownModifier(
                modifier.to_string(),
            ));
        }
    }

    // Error for any remaining characters
    let remaining = chars.join("");
    if !remaining.is_empty() {
        return Err(ParseStandardTwistError::UnexpectedValue(remaining));
    }

    // Apply modifiers
    if inverse {
        direction = direction.reverse();
    }
    if double {
        direction = direction
            .double()
            .ok_or(ParseStandardTwistError::TwistDoesNothing(format!(
                "{}2",
                twist_to_standard_string(&Twist::new(face, direction, layer))
            )))?
    }

    Ok(Twist::new(face, direction, layer))
}

/// Creates a twist from its MC4D notation
fn parse_mc4d_twist_string(s: &str) -> Result<Twist, ParseMC4DTwistError> {
    use once_cell::sync::Lazy;
    use ParseMC4DTwistError::*;

    static MC4D_TWISTS: Lazy<Vec<Option<(Face, TwistDirection)>>> =
        Lazy::new(|| mc4d_twist_order().collect());

    let mut segments = s.split(',');

    let twist_id_string = segments.next().ok_or(MissingId)?.to_owned();

    let twist_id = twist_id_string
        .parse::<usize>()
        .or(Err(InvalidId(twist_id_string.clone())))?;

    let (face, direction) = MC4D_TWISTS
        .get(twist_id)
        .ok_or(InvalidId(twist_id_string.clone()))?
        .ok_or(InvalidId(twist_id_string))?;

    let twist_amount_string = segments.next().ok_or(MissingAmount)?.to_owned();

    let direction = match twist_amount_string
        .parse::<i8>()
        .or(Err(InvalidAmount(twist_amount_string.clone())))?
    {
        1 => direction,
        2 => direction
            .double()
            .ok_or(InvalidAmount(twist_amount_string.clone()))?,
        -1 => direction.reverse(),
        -2 => direction
            .reverse()
            .double()
            .ok_or(InvalidAmount(twist_amount_string.clone()))?,
        _ => return Err(ParseMC4DTwistError::InvalidAmount(twist_amount_string)),
    };

    let slice_mask_string = segments.next().ok_or(MissingSliceMask)?.to_owned();

    let layer = Layer::try_from_primitive(
        slice_mask_string
            .parse()
            .or(Err(InvalidSliceMask(slice_mask_string.clone())))?,
    )
    .or(Err(InvalidSliceMask(slice_mask_string)))?;

    let next_string = segments.next();
    if let Some(value) = next_string {
        return Err(UnexpectedValue(value.to_owned()));
    }
    Ok(Twist::new(face, direction, layer))
}

pub fn mc4d_twist_order() -> impl Iterator<Item = Option<(Face, TwistDirection)>> {
    const MC4D_FACE_ORDER: [Face; 8] = [
        Face::I,
        Face::B,
        Face::D,
        Face::L,
        Face::R,
        Face::U,
        Face::F,
        Face::O,
    ];

    let piece_locations =
        itertools::iproduct!([-1, 0, 1], [-1, 0, 1], [-1, 0, 1]).map(|(x, y, z)| [x, y, z]);
    let corners = piece_locations
        .clone()
        .filter(|v| Vector3::from(*v).magnitude_squared() == 3);
    let edges = piece_locations
        .clone()
        .filter(|v| Vector3::from(*v).magnitude_squared() == 2);
    let centers = piece_locations.filter(|v| Vector3::from(*v).magnitude_squared() == 1);
    let core = std::iter::once([0, 0, 0]);
    let mc4d_order_piece_locations = corners.chain(edges).chain(centers).chain(core);

    MC4D_FACE_ORDER.into_iter().flat_map(move |face| {
        let mut basis = face.basis_faces();
        basis.sort_by_key(|f| f.axis()); // order: X, Y, Z, W
        basis.reverse(); // order: W, Z, Y, X

        mc4d_order_piece_locations
            .clone()
            .map(move |mc4d_coords_of_sticker_within_face: [i32; 3]| {
                let mut offset = Vector4::from_elem(0);
                for i in 0..3 {
                    offset[basis[i].axis() as usize] += mc4d_coords_of_sticker_within_face[i];
                }

                TwistDirection::from_signs_within_face(signs_within_face(face, -offset))
            })
            .map(move |twist_dir| Some((face, twist_dir?)))
    })
}

fn signs_within_face(face: Face, piece_loc_signs: Vector4<i32>) -> Vector3<i32> {
    let [basis1, basis2, basis3] = face.basis();
    vector!(
        piece_loc_signs.dot(basis1),
        piece_loc_signs.dot(basis2),
        piece_loc_signs.dot(basis3)
    )
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_twist_standard_string() {
        let twist = Twist::new(Face::I, TwistDirection::R, Layer::This);
        assert_eq!(twist_to_standard_string(&twist), "IR");

        let twist = Twist::new(Face::I, TwistDirection::R, Layer::Other);
        assert_eq!(twist_to_standard_string(&twist), "OL");

        let twist = Twist::new(Face::R, TwistDirection::F, Layer::This);
        assert_eq!(twist_to_standard_string(&twist), "RF");

        let twist = Twist::new(Face::R, TwistDirection::F, Layer::Both);
        assert_eq!(twist_to_standard_string(&twist), "{1-2}RF");

        let twist = Twist::new(Face::B, TwistDirection::UFR, Layer::This);
        assert_eq!(twist_to_standard_string(&twist), "BURI");

        let twist = Twist::new(Face::R, TwistDirection::UFR, Layer::This);
        assert_eq!(twist_to_standard_string(&twist), "RUFO");
    }

    #[test]
    fn test_twist_from_standard_string() {
        let twist = Twist::from_str("RU").unwrap();
        let expected = Twist::new(Face::R, TwistDirection::U, Layer::This);
        assert_eq!(twist, expected);

        let twist = Twist::from_str("{2}RU").unwrap();
        let expected = Twist::new(Face::R, TwistDirection::U, Layer::Other);
        assert_eq!(twist, expected);

        let twist = Twist::from_str("{1-2}RU").unwrap();
        let expected = Twist::new(Face::R, TwistDirection::U, Layer::Both);
        assert_eq!(twist, expected);

        let twist = Twist::from_str("{1,2}RU").unwrap();
        let expected = Twist::new(Face::R, TwistDirection::U, Layer::Both);
        assert_eq!(twist, expected);

        let twist = Twist::from_str("RUFO").unwrap();
        let expected = Twist::new(Face::R, TwistDirection::UFR, Layer::This);
        assert_eq!(twist, expected);

        let twist = Twist::from_str("BURO").unwrap();
        let expected = Twist::new(Face::B, TwistDirection::UBR, Layer::This);
        assert_eq!(twist, expected);

        let twist = Twist::from_str("DIRF").unwrap();
        let expected = Twist::new(Face::D, TwistDirection::UFR, Layer::This);
        assert_eq!(twist, expected);

        let twist = Twist::from_str("IFL").unwrap();
        let expected = Twist::new(Face::I, TwistDirection::FL, Layer::This);
        assert_eq!(twist, expected);

        let twist = Twist::from_str("IFL'").unwrap();
        let expected = Twist::new(Face::I, TwistDirection::FL.reverse(), Layer::This);
        assert_eq!(twist, expected);

        let twist = Twist::from_str("RUFO2").unwrap();
        let expected = Twist::new(Face::R, TwistDirection::UFR.double().unwrap(), Layer::This);
        assert_eq!(twist, expected);

        let twist = Twist::from_str("ULO").unwrap();
        let expected = Twist::new(Face::U, TwistDirection::UL, Layer::This);
        assert_eq!(twist, expected);
    }

    #[test]
    fn test_twist_to_from_standard_string() {
        Twist::iter().for_each(|twist| {
            assert_eq!(
                twist,
                parse_standard_twist_string(twist_to_standard_string(&twist).as_str()).unwrap()
            )
        })
    }
}
