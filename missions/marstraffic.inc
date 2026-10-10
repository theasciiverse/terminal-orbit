/*
 *  Mars traffic: Deimos Depot and a tanker running between it and the
 *  Mars Science Station.  Needs marssci.inc first.
 */
Cursor { Deimos +2000 +0 +1500 }
Object { Name DeimosDepot Model station2.tri Friendly MaxShields 400 }
Cursor { DeimosDepot -2500 +0 +0 }
Object { Name Tanker1 Model freight1.tri Friendly Strategy Ferry Route { MarsSci DeimosDepot } Cruise 1200 }
