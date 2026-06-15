/*
 *  Squadron of three friendly fighters
 */

Weapon
{
	Index 8
	Yield 30
	Color 0x00ff00
	Renderer 2
}

Cursor { +0 +3000 +0 }
Object { Name Alpha1 Model fight1.tri Friendly Strategy hunt3 }
Cursor { +0 -3000 += }

Cursor { -2000 +2000 -2000 }
Object { Name Alpha2 Model fight1.tri Friendly Strategy hunt4 Weapon 8 }
Cursor { +2000 -2000 +2000 }

Cursor { +2000 -2000 -2000 }
Object { Name Alpha3 Model fight1.tri Friendly Strategy hunt4 }
Cursor { -2000 +2000 +2000 }

