use crate::constants::*;
use crate::physics::position_planets;
use crate::types::*;

/// Raw planet definition before unit conversion.
struct PlanetDef {
    name: &'static str,
    radius_km: f64,
    /// Distance in compressed mode (roughly 1/10 real), in millions of km.
    dist_compressed: f64,
    /// Distance in real-scale mode, in millions of km.
    dist_real: f64,
    is_moon: bool,
    primary: usize,
    oblicity: f64,
    /// Angular velocity coefficient: 0.004166 / orbital_period_days.
    angvel: f64,
}

const PLANET_DEFS: [PlanetDef; 32] = [
    PlanetDef { name: "Sol",       radius_km: 696_000.0, dist_compressed: 0.0,    dist_real: 0.0,    is_moon: false, primary: 0,  oblicity: 0.0,   angvel: 0.0 },
    PlanetDef { name: "Mercury",   radius_km:   2_440.0, dist_compressed: 5.79,   dist_real: 50.79,  is_moon: false, primary: 0,  oblicity: 0.0,   angvel: 0.004166 / 87.969 },
    PlanetDef { name: "Venus",     radius_km:   6_052.0, dist_compressed: 10.82,  dist_real: 108.2,  is_moon: false, primary: 0,  oblicity: 177.3, angvel: 0.004166 / 224.7 },
    PlanetDef { name: "Earth",     radius_km:   6_371.0, dist_compressed: 14.96,  dist_real: 149.6,  is_moon: false, primary: 0,  oblicity: 23.45, angvel: 0.004166 / 365.256 },
    PlanetDef { name: "Moon",      radius_km:   1_737.5, dist_compressed: 0.3844, dist_real: 0.3844, is_moon: true,  primary: 3,  oblicity: 0.0,   angvel: 0.004166 / 27.3 },
    PlanetDef { name: "Mars",      radius_km:   3_390.0, dist_compressed: 22.79,  dist_real: 227.9,  is_moon: false, primary: 0,  oblicity: 25.19, angvel: 0.004166 / 686.98 },
    PlanetDef { name: "Phobos",    radius_km:      13.0, dist_compressed: 0.00938, dist_real: 0.00938, is_moon: true, primary: 5, oblicity: 0.0,   angvel: 0.004166 / 0.319 },
    PlanetDef { name: "Deimos",    radius_km:       8.0, dist_compressed: 0.02234, dist_real: 0.02234, is_moon: true, primary: 5, oblicity: 0.0,   angvel: 0.004166 / 1.262 },
    PlanetDef { name: "Jupiter",   radius_km:  69_911.0, dist_compressed: 50.0,   dist_real: 778.4,  is_moon: false, primary: 0,  oblicity: 3.12,  angvel: 0.004166 / 4332.0 },
    PlanetDef { name: "Io",        radius_km:   1_821.0, dist_compressed: 0.421,  dist_real: 0.421,  is_moon: true,  primary: 8,  oblicity: 0.0,   angvel: 0.004166 / 1.769 },
    PlanetDef { name: "Europa",    radius_km:   1_565.0, dist_compressed: 0.671,  dist_real: 0.671,  is_moon: true,  primary: 8,  oblicity: 0.0,   angvel: 0.004166 / 3.552 },
    PlanetDef { name: "Ganymede",  radius_km:   2_634.0, dist_compressed: 1.070,  dist_real: 1.070,  is_moon: true,  primary: 8,  oblicity: 0.0,   angvel: 0.004166 / 7.155 },
    PlanetDef { name: "Callisto",  radius_km:   2_403.0, dist_compressed: 1.883,  dist_real: 1.883,  is_moon: true,  primary: 8,  oblicity: 0.0,   angvel: 0.004166 / 16.69 },
    PlanetDef { name: "Saturn",    radius_km:  58_232.0, dist_compressed: 75.0,   dist_real: 1427.0, is_moon: false, primary: 0,  oblicity: 26.73, angvel: 0.004166 / 10759.0 },
    PlanetDef { name: "Mimas",     radius_km:     198.8, dist_compressed: 0.185,  dist_real: 0.185,  is_moon: true,  primary: 13, oblicity: 0.0,   angvel: 0.004166 / 0.942 },
    PlanetDef { name: "Enceladus", radius_km:     249.1, dist_compressed: 0.238,  dist_real: 0.238,  is_moon: true,  primary: 13, oblicity: 0.0,   angvel: 0.004166 / 1.37 },
    PlanetDef { name: "Tethys",    radius_km:     529.9, dist_compressed: 0.294,  dist_real: 0.294,  is_moon: true,  primary: 13, oblicity: 0.0,   angvel: 0.004166 / 1.888 },
    PlanetDef { name: "Dione",     radius_km:     560.0, dist_compressed: 0.377,  dist_real: 0.377,  is_moon: true,  primary: 13, oblicity: 0.0,   angvel: 0.004166 / 2.737 },
    PlanetDef { name: "Rhea",      radius_km:     764.0, dist_compressed: 0.527,  dist_real: 0.527,  is_moon: true,  primary: 13, oblicity: 0.0,   angvel: 0.004166 / 4.518 },
    PlanetDef { name: "Titan",     radius_km:   2_575.0, dist_compressed: 1.221,  dist_real: 1.221,  is_moon: true,  primary: 13, oblicity: 0.0,   angvel: 0.004166 / 15.9 },
    PlanetDef { name: "Iapetus",   radius_km:     718.0, dist_compressed: 3.561,  dist_real: 3.561,  is_moon: true,  primary: 13, oblicity: 0.0,   angvel: 0.004166 / 79.33 },
    PlanetDef { name: "Uranus",    radius_km:  25_362.0, dist_compressed: 100.0,  dist_real: 2871.0, is_moon: false, primary: 0,  oblicity: 97.86, angvel: 0.004166 / 30685.0 },
    PlanetDef { name: "Miranda",   radius_km:     235.0, dist_compressed: 0.130,  dist_real: 0.130,  is_moon: true,  primary: 21, oblicity: 0.0,   angvel: 0.004166 / 1.413 },
    PlanetDef { name: "Ariel",     radius_km:     580.0, dist_compressed: 0.191,  dist_real: 0.191,  is_moon: true,  primary: 21, oblicity: 0.0,   angvel: 0.004166 / 2.52 },
    PlanetDef { name: "Umbriel",   radius_km:     585.0, dist_compressed: 0.266,  dist_real: 0.266,  is_moon: true,  primary: 21, oblicity: 0.0,   angvel: 0.004166 / 4.144 },
    PlanetDef { name: "Titania",   radius_km:     789.0, dist_compressed: 0.436,  dist_real: 0.436,  is_moon: true,  primary: 21, oblicity: 0.0,   angvel: 0.004166 / 8.706 },
    PlanetDef { name: "Oberon",    radius_km:     761.0, dist_compressed: 0.583,  dist_real: 0.583,  is_moon: true,  primary: 21, oblicity: 0.0,   angvel: 0.004166 / 13.463 },
    PlanetDef { name: "Neptune",   radius_km:  24_766.0, dist_compressed: 125.0,  dist_real: 4498.0, is_moon: false, primary: 0,  oblicity: 29.56, angvel: 0.004166 / 60189.0 },
    PlanetDef { name: "Triton",    radius_km:   1_352.6, dist_compressed: 0.355,  dist_real: 0.355,  is_moon: true,  primary: 27, oblicity: 0.0,   angvel: 0.004166 / -5.877 },
    PlanetDef { name: "Proteus",   radius_km:     218.0, dist_compressed: 0.118,  dist_real: 0.118,  is_moon: true,  primary: 27, oblicity: 0.0,   angvel: 0.004166 / 1.122 },
    PlanetDef { name: "Pluto",     radius_km:   1_137.0, dist_compressed: 150.0,  dist_real: 5906.0, is_moon: false, primary: 0,  oblicity: 122.0, angvel: 0.004166 / 90465.0 },
    PlanetDef { name: "Charon",    radius_km:     586.0, dist_compressed: 0.019,  dist_real: 0.019,  is_moon: true,  primary: 30, oblicity: 0.0,   angvel: 0.004166 / 6.387 },
];

/// Populate world.planets from the built-in solar system data.
/// Mirrors ResetPlanets() + PositionPlanets() from planet.c.
/// Call this after World::new(). Planets start at theta=0.
pub fn reset_planets(world: &mut World) {
    for (p, def) in PLANET_DEFS.iter().enumerate() {
        let dist_raw = if world.real_distances { def.dist_real } else { def.dist_compressed };
        let radius = def.radius_km / KM_TO_UNITS1;
        let dist   = dist_raw / KM_TO_UNITS2;

        world.planets[p].name      = def.name.to_string();
        world.planets[p].radius    = radius;
        world.planets[p].radius2   = radius * radius;
        world.planets[p].dist      = dist;
        world.planets[p].is_moon   = def.is_moon;
        world.planets[p].primary   = def.primary;
        world.planets[p].oblicity  = def.oblicity;
        world.planets[p].angvel    = def.angvel;
        world.planets[p].mass      = radius * radius * radius;
        world.planets[p].hidden    = false;
        world.planets[p].custom    = true;
        // theta = 0 (fixed starting angle; Phase 9 adds randomisation)
        world.planets[p].theta     = 0.0;
    }
    position_planets(world);
}

/// Populate world.weapons from built-in data. Mirrors InitWeapons() from weapon.c.
pub fn init_weapons(world: &mut World) {
    let w = &mut world.weapons;

    w[0].name = "Laser".to_string();
    w[0].speed = 4_000.0 / KM_TO_UNITS1;
    w[0].damage = 20.0;
    w[0].idle = 0.2;
    w[0].expire = 1.0;
    w[0].renderer = 1;
    w[0].color = [1.0, 1.0, 0.0];

    w[1].name = "PhotonRay".to_string();
    w[1].speed = 5_000.0 / KM_TO_UNITS1;
    w[1].damage = 30.0;
    w[1].idle = 0.3;
    w[1].expire = 0.75;
    w[1].renderer = 2;
    w[1].color = [0.0, 0.0, 1.0];

    w[2].name = "IonGun".to_string();
    w[2].speed = 4_000.0 / KM_TO_UNITS1;
    w[2].damage = 30.0;
    w[2].idle = 0.5;
    w[2].expire = 1.5;
    w[2].renderer = 3;
    w[2].color = [0.0, 1.0, 0.0];

    w[3].name = "Disruptor".to_string();
    w[3].speed = 3_000.0 / KM_TO_UNITS1;
    w[3].damage = 60.0;
    w[3].idle = 1.0;
    w[3].expire = 1.5;
    w[3].renderer = 4;
    w[3].color = [1.0, 1.0, 0.0];

    for i in NPLAYER_WEAPONS..NWEAPONS {
        w[i].name = "Spare".to_string();
        w[i].speed = 6_000.0 / KM_TO_UNITS1;
        w[i].damage = 15.0;
        w[i].idle = 2.0;
        w[i].expire = 1.0;
        w[i].renderer = 0;
        w[i].color = [1.0, 0.0, 0.0];
    }

    // range2 = (speed * expire)^2
    for i in 0..NWEAPONS {
        let r = w[i].speed * w[i].expire;
        w[i].range2 = r * r;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::World;

    #[test]
    fn planets_initialized() {
        let mut world = World::new();
        reset_planets(&mut world);

        assert_eq!(world.planets[0].name, "Sol");
        assert_eq!(world.planets[3].name, "Earth");
        assert_eq!(world.planets[30].name, "Pluto");
        assert_eq!(world.planets[31].name, "Charon");
    }

    #[test]
    fn earth_radius_in_game_units() {
        let mut world = World::new();
        reset_planets(&mut world);
        // Earth radius: 6371 km / 6000 ≈ 1.062
        let r = world.planets[3].radius;
        assert!((r - 6371.0 / 6000.0).abs() < 1e-6);
    }

    #[test]
    fn moon_is_moon_of_earth() {
        let mut world = World::new();
        reset_planets(&mut world);
        assert!(world.planets[4].is_moon);
        assert_eq!(world.planets[4].primary, 3);
    }

    #[test]
    fn planet_mass_is_radius_cubed() {
        let mut world = World::new();
        reset_planets(&mut world);
        for p in &world.planets {
            let expected = p.radius * p.radius * p.radius;
            assert!((p.mass - expected).abs() < 1e-12);
        }
    }

    #[test]
    fn weapons_initialized() {
        let mut world = World::new();
        init_weapons(&mut world);
        assert_eq!(world.weapons[0].name, "Laser");
        assert_eq!(world.weapons[3].name, "Disruptor");
        assert_eq!(world.weapons[NPLAYER_WEAPONS].name, "Spare");
        // range2 must be positive
        for w in &world.weapons {
            assert!(w.range2 > 0.0);
        }
    }
}
