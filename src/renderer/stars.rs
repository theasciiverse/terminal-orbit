use crate::math::Vec3;
use crate::star_data::STARS;

use super::canvas::BrailleCanvas;
use super::projection::Camera;

/// Return all star unit-vectors with their visual magnitudes.
pub fn all_stars() -> Vec<(Vec3, f32)> {
    STARS.iter().map(|s| (Vec3::new(s[0] as f64, s[1] as f64, s[2] as f64), s[3])).collect()
}

/// Draw stars split across three magnitude-band canvases.
///   c_bright : mag < 1.5  → rendered white
///   c_medium : 1.5–3.5    → rendered light gray
///   c_dim    : 3.5–5.5    → rendered dark gray (only in dense mode)
/// Bright stars get a 2×2 braille cluster; mag 1.5–2.5 get a 2×1 pair; dimmer = single dot.
pub fn draw_stars(
    c_bright: &mut BrailleCanvas,
    c_medium: &mut BrailleCanvas,
    c_dim:    &mut BrailleCanvas,
    camera:   &Camera,
    stars:    &[(Vec3, f32)],
    dense:    bool,
) {
    for &(star, mag) in stars {
        if !dense && mag >= 4.5 { continue; }

        let Some((px, py)) = camera.project_direction(star) else { continue };

        if mag < 1.5 {
            // Very bright — 2×2 cluster so it pops.
            c_bright.set(px, py);
            c_bright.set(px + 1, py);
            c_bright.set(px, py + 1);
            c_bright.set(px + 1, py + 1);
        } else if mag < 2.5 {
            // Bright — 2×1 pair.
            c_medium.set(px, py);
            c_medium.set(px + 1, py);
        } else if mag < 3.5 {
            // Medium — single dot.
            c_medium.set(px, py);
        } else {
            // Dim — single dot, dark gray.
            c_dim.set(px, py);
        }
    }
}

// ─── Constellation lines ─────────────────────────────────────────────────────

/// (RA decimal hours, Dec degrees) → unit Vec3 matching the stars.h coordinate system.
fn radec(ra_h: f32, dec_deg: f32) -> Vec3 {
    let ra  = ra_h * std::f32::consts::PI / 12.0;
    let dec = dec_deg * std::f32::consts::PI / 180.0;
    Vec3::new(
        (dec.cos() * ra.cos()) as f64,
        (dec.cos() * ra.sin()) as f64,
        dec.sin() as f64,
    )
}

/// Draw IAU stick-figure constellation lines onto a braille canvas.
/// The canvas should be rendered in a very dim color (e.g. Color::Rgb(45,55,95)).
pub fn draw_constellation_lines(canvas: &mut BrailleCanvas, camera: &Camera) {
    for &((ra1, dec1), (ra2, dec2)) in CONSTELLATION_LINES {
        let a = radec(ra1, dec1);
        let b = radec(ra2, dec2);
        if let (Some(p0), Some(p1)) = (
            camera.project_direction(a),
            camera.project_direction(b),
        ) {
            canvas.line(p0.0, p0.1, p1.0, p1.1);
        }
    }
}

/// Name, label position (RA hours, Dec degrees) for each constellation.
/// The position is near the centroid of the stick figure.
pub const CONSTELLATION_LABELS: &[(&str, f32, f32)] = &[
    ("Orion",        5.60,  2.0),
    ("Ursa Major",  12.60, 56.0),
    ("Cassiopeia",   1.00, 60.0),
    ("Cygnus",      20.40, 41.0),
    ("Scorpius",    16.90, -30.0),
    ("Leo",         10.80, 15.0),
    ("Gemini",       7.10, 24.0),
    ("Perseus",      3.50, 45.0),
    ("Auriga",       5.60, 40.0),
    ("Taurus",       5.00, 20.0),
    ("Virgo",       13.10, -2.0),
    ("Boötes",      14.40, 22.0),
    ("Andromeda",    1.00, 36.0),
    ("Pegasus",     22.70, 20.0),
];

/// Major constellation stick-figure lines as ((RA_h, Dec°), (RA_h, Dec°)) pairs.
/// Positions from the Yale Bright Star Catalogue (approximate).
const CONSTELLATION_LINES: &[((f32, f32), (f32, f32))] = &[
    // ── Orion ──────────────────────────────────────────────────────────────
    // Shoulders
    ((5.919, 7.407),   (5.419, 6.350)),   // Betelgeuse – Bellatrix
    // Head
    ((5.919, 7.407),   (5.585, 9.934)),   // Betelgeuse – Meissa
    ((5.419, 6.350),   (5.585, 9.934)),   // Bellatrix  – Meissa
    // Shoulders to belt
    ((5.919, 7.407),   (5.533, -0.299)),  // Betelgeuse – Mintaka
    ((5.419, 6.350),   (5.533, -0.299)),  // Bellatrix  – Mintaka
    // Belt
    ((5.533, -0.299),  (5.604, -1.202)),  // Mintaka – Alnilam
    ((5.604, -1.202),  (5.679, -1.943)),  // Alnilam – Alnitak
    // Belt to feet
    ((5.679, -1.943),  (5.242, -8.202)),  // Alnitak – Rigel
    ((5.679, -1.943),  (5.797, -9.670)),  // Alnitak – Saiph
    ((5.242, -8.202),  (5.797, -9.670)),  // Rigel   – Saiph

    // ── Ursa Major (Big Dipper) ────────────────────────────────────────────
    ((13.792, 49.313), (13.399, 54.926)), // Alkaid  – Mizar
    ((13.399, 54.926), (12.900, 55.960)), // Mizar   – Alioth
    ((12.900, 55.960), (12.257, 57.033)), // Alioth  – Megrez
    ((12.257, 57.033), (11.897, 53.695)), // Megrez  – Phecda
    ((11.897, 53.695), (11.031, 56.383)), // Phecda  – Merak
    ((11.031, 56.383), (11.062, 61.751)), // Merak   – Dubhe
    ((11.062, 61.751), (12.257, 57.033)), // Dubhe   – Megrez (bowl)

    // ── Cassiopeia (the W) ─────────────────────────────────────────────────
    ((0.153,  59.151), (0.675,  56.537)), // Caph    – Schedar
    ((0.675,  56.537), (0.945,  60.717)), // Schedar – Gamma
    ((0.945,  60.717), (1.430,  60.235)), // Gamma   – Ruchbah
    ((1.430,  60.235), (1.907,  63.670)), // Ruchbah – Segin

    // ── Cygnus (Northern Cross) ────────────────────────────────────────────
    ((19.512, 27.960), (20.370, 40.257)), // Albireo  – Sadr
    ((20.370, 40.257), (20.691, 45.280)), // Sadr     – Deneb
    ((19.749, 45.131), (20.370, 40.257)), // Fawaris  – Sadr
    ((20.370, 40.257), (20.770, 33.970)), // Sadr     – Gienah

    // ── Scorpius ───────────────────────────────────────────────────────────
    ((16.091, -19.805), (16.005, -22.622)), // Graffias – Dschubba
    ((16.005, -22.622), (16.352, -25.593)), // Dschubba – Alniyat
    ((16.352, -25.593), (16.490, -26.432)), // Alniyat  – Antares
    ((16.490, -26.432), (16.598, -28.216)), // Antares  – Tau
    ((16.598, -28.216), (16.836, -34.293)), // Tau      – mid-body
    ((16.836, -34.293), (17.622, -42.998)), // mid      – Sargas
    ((17.622, -42.998), (17.560, -37.104)), // Sargas   – Shaula
    ((17.560, -37.104), (17.531, -37.296)), // Shaula   – Lesath

    // ── Leo (the Sickle) ──────────────────────────────────────────────────
    ((10.139, 11.967),  (10.333, 19.841)), // Regulus  – Algieba
    ((10.333, 19.841),  (10.278, 23.417)), // Algieba  – Adhafera
    ((10.278, 23.417),  (10.123, 23.774)), // Adhafera – Rasalas
    ((10.123, 23.774),  (9.765,  26.007)), // Rasalas  – Algenubi (head)
    ((9.765,  26.007),  (10.139, 11.967)), // Algenubi – Regulus (close sickle)
    ((10.139, 11.967),  (11.237, 15.430)), // Regulus  – Chertan (body)
    ((11.237, 15.430),  (11.235, 20.524)), // Chertan  – Zosma
    ((11.235, 20.524),  (10.333, 19.841)), // Zosma    – Algieba (back)
    ((11.235, 20.524),  (11.817, 14.572)), // Zosma    – Denebola (tail)

    // ── Gemini ─────────────────────────────────────────────────────────────
    ((7.577, 31.888),  (7.755, 28.026)),   // Castor  – Pollux
    ((7.577, 31.888),  (7.335, 21.982)),   // Castor  – Wasat
    ((7.335, 21.982),  (6.629, 16.399)),   // Wasat   – Alhena
    ((7.755, 28.026),  (7.741, 24.398)),   // Pollux  – Kappa
    ((7.741, 24.398),  (6.629, 16.399)),   // Kappa   – Alhena
    ((7.577, 31.888),  (6.289, 22.506)),   // Castor  – Propus (other arm)
    ((7.755, 28.026),  (6.383, 22.514)),   // Pollux  – Tejat

    // ── Perseus ────────────────────────────────────────────────────────────
    ((3.405, 49.861),  (3.136, 40.957)),   // Mirfak – Algol
    ((3.405, 49.861),  (3.080, 53.506)),   // Mirfak – Gamma Per
    ((3.405, 49.861),  (3.963, 31.884)),   // Mirfak – Atik
    ((3.963, 31.884),  (3.987, 35.791)),   // Atik   – Menkib

    // ── Auriga ─────────────────────────────────────────────────────────────
    ((5.278, 45.998),  (5.992, 44.947)),   // Capella    – Menkalinan
    ((5.992, 44.947),  (5.995, 37.212)),   // Menkalinan – Mahasim
    ((5.995, 37.212),  (4.950, 33.166)),   // Mahasim    – Hassaleh
    ((4.950, 33.166),  (5.438, 28.608)),   // Hassaleh   – El Nath
    ((5.438, 28.608),  (5.278, 45.998)),   // El Nath    – Capella

    // ── Taurus ─────────────────────────────────────────────────────────────
    ((4.599, 16.509),  (5.438, 28.608)),   // Aldebaran – El Nath
    ((4.599, 16.509),  (5.627, 21.143)),   // Aldebaran – Alheka

    // ── Virgo ──────────────────────────────────────────────────────────────
    ((13.420, -11.161), (12.694, -1.450)), // Spica      – Porrima
    ((12.694, -1.450),  (12.926,  3.398)), // Porrima    – Auva
    ((12.926,  3.398),  (12.694, 10.959)), // Auva       – Vindemiatrix
    ((11.845,  1.765),  (12.694, -1.450)), // Zavijava   – Porrima

    // ── Boötes ─────────────────────────────────────────────────────────────
    ((14.261, 19.182),  (14.749, 27.074)), // Arcturus  – Nekkar
    ((14.261, 19.182),  (14.534, 30.371)), // Arcturus  – Seginus
    ((14.261, 19.182),  (14.530, 13.724)), // Arcturus  – Izar
    ((14.530, 13.724),  (14.686,  16.418)), // Izar     – Muphrid
    ((14.686, 16.418),  (14.261, 19.182)), // Muphrid  – Arcturus (close)

    // ── Andromeda ──────────────────────────────────────────────────────────
    ((0.139, 29.091),  (1.162, 35.621)),   // Alpheratz – Mirach
    ((1.162, 35.621),  (1.633, 41.406)),   // Mirach    – Almach
    ((1.162, 35.621),  (0.655, 30.861)),   // Mirach    – Delta And

    // ── Pegasus (Great Square) ─────────────────────────────────────────────
    ((0.139,  29.091), (23.079, 15.206)),  // Alpheratz – Markab
    ((23.079, 15.206), (22.691, 10.832)), // Markab    – Sadalbari
    ((22.691, 10.832), (23.063, 28.083)), // Sadalbari – Scheat (approx)
    ((23.063, 28.083), (0.139,  29.091)), // Scheat    – Alpheratz
];
