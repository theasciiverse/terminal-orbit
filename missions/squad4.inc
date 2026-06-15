/*
 *  Squadron of 4 misc fighters
 */

Cursor { +0 +0 +3000 }
Object { Name Gamma1 Model light1.tri Strategy hunt3 }
Cursor { +0 +0 -3000 }

Cursor { -2000 -2000 -2000 }
Object { Name Gamma2 Model light1.tri Strategy hunt3 }
Cursor { +2000 +2000 +2000 }

Cursor { +2000 -2000 -2000 }
Object { Name Gamma3 Model light2.tri Strategy hunt4 }
Cursor { -2000 +2000 +2000 }

Weapon
{
	Index 9
	Yield 30
	Color 0x00ffff
	Renderer 2
}

Cursor { +0 +2000 -2000 }
Object { Name GammaAce Model heavy1.tri Strategy hunt4 Weapon 9 }
Cursor { +0 -2000 +2000 }

