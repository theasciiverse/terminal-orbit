/*
 *  Jovian traffic: a supply shuttle looping between Europa Station and a
 *  radiation-shielded relay.  Needs europastation.inc first.
 */
Cursor { EuropaStation +0 -25000 +6000 }
Object { Name JoveRelay Model comm1.tri Friendly }
Cursor { JoveRelay +2000 +0 +0 }
Object { Name Shuttle1 Model trans1.tri Friendly Strategy Ferry Route { EuropaStation JoveRelay } Cruise 1500 }
