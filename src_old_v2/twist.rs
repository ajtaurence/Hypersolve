use lazy_static::lazy_static;

use crate::common::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Twist {
    axis: Axis,
    matrix: Matrix,
    layers: Layers,
}

pub enum Layers {
    Neg = 1,
    Pos = 2,
    Both = 3,
}

// https://github.com/HactarCE/Hyperspeedcube/blob/main/src/puzzle/rubiks_4d.rs#L683-L718

// From Hyperspeedcube
// https://github.com/HactarCE/Hyperspeedcube/blob/645bbd3e88eec62d25a22c835a7174a0b2f44f99/src/puzzle/rubiks_4d.rs#L637-L718
impl Twist {
    pub fn to_mc4d_string(mut self) -> String {
        lazy_static! {
            static ref MC4D_TWIST_IDS: HashMap<(TwistAxis, TwistDirection), usize> =
                Twist::mc4d_twist_order()
                    .enumerate()
                    .filter_map(|(i, twist)| Some((twist?, i)))
                    .collect();
        }

        let dir: TwistDirectionEnum = twist.direction.into();
        if let Some(quarter_turn) = dir.half() {
            twist.direction = quarter_turn.into();
            return format!("{0} {0}", Self::to_mc4d_twist_string(twist));
        }
        let sticker_id = MC4D_TWIST_IDS[&(twist.axis, twist.direction)];
        let direction_id = 1;
        let layer_mask = twist.layers.0;
        format!("{sticker_id},{direction_id},{layer_mask}")
    }

    pub fn from_mc4d_string(s: &str) -> Option<Self> {
        lazy_static! {
            static ref MC4D_TWISTS: Vec<Option<(TwistAxis, TwistDirection)>> =
                Twist::mc4d_twist_order().collect();
        }

        let mut segments = s.split(',');

        let (axis, direction) = (*MC4D_TWISTS.get(segments.next()?.parse::<usize>().ok()?)?)?;
        let direction: TwistDirectionEnum = direction.into();
        let direction = match segments.next()?.parse::<i8>().ok()? {
            1 => direction,
            2 => direction.double()?,
            -1 => direction.rev(),
            -2 => direction.rev().double()?,
            _ => return None,
        };
        let layers = LayerMask(segments.next()?.parse().ok()?);
        if segments.next().is_some() {
            return None;
        }
        Some(Twist {
            axis,
            direction: direction.into(),
            layers,
        })
    }

    fn mc4d_twist_order() -> impl Iterator<Item = Option<(TwistAxis, TwistDirection)>> {
        // Intentionally switch I and O
        const MC4D_FACET_ORDER: [Facet; 8] = [
            Facet::O,
            Facet::B,
            Facet::D,
            Facet::L,
            Facet::R,
            Facet::U,
            Facet::F,
            Facet::I,
        ];

        MC4D_FACET_ORDER.into_iter().flat_map(|facet| {
            let mut basis = facet.basis_faces();
            basis.sort_by_key(|f| f.axis); // order: X, Y, Z, W
            basis.reverse(); // order: W, Z, Y, X
            let mc4d_basis_1 = basis[0].axis;
            let mc4d_basis_2 = basis[1].axis;
            let mc4d_basis_3 = basis[2].axis;

            let piece_locations =
                itertools::iproduct!([-1, 0, 1], [-1, 0, 1], [-1, 0, 1]).map(|(x, y, z)| [x, y, z]);
            let corners = piece_locations.clone().filter(|v| v.magnitude2() == 3);
            let edges = piece_locations.clone().filter(|v| v.magnitude2() == 2);
            let centers = piece_locations.filter(|v| v.magnitude2() == 1);
            let core = std::iter::once([0, 0, 0]);
            let mc4d_order_piece_locations = corners.chain(edges).chain(centers).chain(core);

            mc4d_order_piece_locations
                .map(move |mc4d_coords_of_sticker_within_facet: [i8; 3]| {
                    let mut offset = [0; 4];
                    for i in 0..3 {
                        offset[basis[i].axis as usize] += mc4d_coords_of_sticker_within_facet[i];
                    }

                    TwistDirectionEnum::from_signs_within_face(Self::signs_within_face(
                        facet,
                        match facet {
                            O => offset, // not sure why this is necessary, but it is
                            _ => -offset,
                        },
                    ))
                })
                .map(move |twist_dir| Some((face.into(), twist_dir?.into())))
        })
    }
}

fn axis_angle_matrix(axis: [i8; 3]) -> Option<[Facet; 3]> {
    let mag_sq: i8 = axis.into_iter().map(|x| x * x).sum();

    // Determine the period of the twist based on
    // which 3^4 piece type we are twisting around
    let period = match axis.into_iter().map(i8::abs).sum() {
        1 => 4,
        2 => 2,
        3 => 3,
        _ => return None,
    };

    // Let theta = 2*PI/period. Compute 4cos²(θ/2) and 4sin²(θ/2)
    let (cos_sq_half_theta_times_4, sin_sq_half_theta_times_4) = match period {
        4 => (2, 2),
        2 => (0, 4),
        3 => (1, 3),
        _ => panic!("bad twist period"),
    };
    // Compute 2sin(theta)÷||axis||
    let sin_theta_times_2 = match period {
        4 => 2, // scaled by sqrt(1)
        2 => 0, // scaled by sqrt(2)
        3 => 1, // scaled by sqrt(3)
        _ => panic!("bad twist period"),
    };

    /// https://en.wikipedia.org/wiki/Levi-Civita_symbol#Three_dimensions
    fn levi_civita(j: usize, k: usize) -> i8 {
        // Infer third parameter based on the first two
        match (j, k) {
            (0, 1) | (1, 2) | (2, 0) => 1,  // even permutation
            (2, 1) | (0, 2) | (1, 0) => -1, // odd permutation
            _ => 0,                         // duplicate
        }
    }

    // https://en.wikipedia.org/wiki/Rotation_matrix
    let mut ret = [[0; 3]; 3];
    for k in 0..3 {
        for j in 0..3 {
            ret[j][k] = if j == k {
                cos_sq_half_theta_times_4 * mag_sq
                    + sin_sq_half_theta_times_4 * (2 * axis[j] * axis[j] - 1)
            } else {
                2 * axis[j] * axis[k] * sin_sq_half_theta_times_4
                    - 2 * levi_civita(j, k) * sin_theta_times_2
            };
        }
    }
}
