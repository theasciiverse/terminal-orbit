/*
 *  Titan traffic: an ice tug looping between Titan Base and a fuel
 *  depot.  Needs titanbase.inc first.
 */
Cursor { TitanBase +0 +20000 -4000 }
Object { Name KrakenDepot Model station2.tri Friendly MaxShields 400 }
Cursor { KrakenDepot +2000 +0 +0 }
Object { Name IceTug1 Model freight1.tri Friendly Strategy Ferry Route { TitanBase KrakenDepot } Cruise 1200 }
