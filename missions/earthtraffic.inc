/*
 *  Cislunar traffic: two freighters shuttling between Earth Base and
 *  Shackleton Station.  Needs earthbase.inc and shackleton.inc first.
 */
Cursor { EarthBase +3000 +2000 +0 }
Object { Name Hauler1 Model freight1.tri Friendly Strategy Ferry Route { Shackleton EarthBase } Cruise 1500 }
Cursor { Shackleton -3000 +2000 +0 }
Object { Name Hauler2 Model trans1.tri Friendly Strategy Ferry Route { EarthBase Shackleton } Cruise 1200 }
