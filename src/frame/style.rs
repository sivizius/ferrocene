use crate::
{
  Flags,
};

bitflags!
{
  pub struct Style: Flags
  {
    const None                          =                                       0b0000_0000_0000_0000_0000_0000_0000_0000;
    const Italic                        =                                       0b0000_0000_0000_0000_0000_0000_0000_0001;
    const Underline                     =                                       0b0000_0000_0000_0000_0000_0000_0000_0010;
    const SlowBlink                     =                                       0b0000_0000_0000_0000_0000_0000_0000_0100;
    const RapidBlink                    =                                       0b0000_0000_0000_0000_0000_0000_0000_1000;
    const Inverse                       =                                       0b0000_0000_0000_0000_0000_0000_0001_0000;
    const Conceal                       =                                       0b0000_0000_0000_0000_0000_0000_0010_0000;
    const CrossedOut                    =                                       0b0000_0000_0000_0000_0000_0000_0100_0000;
    const Fraktur                       =                                       0b0000_0000_0000_0000_0000_0000_1000_0000;
    const DoubleUnderline               =                                       0b0000_0000_0000_0000_0000_0001_0000_0000;
    const Framed                        =                                       0b0000_0000_0000_0000_0000_0010_0000_0000;
    const Encircled                     =                                       0b0000_0000_0000_0000_0000_0100_0000_0000;
    const Overlined                     =                                       0b0000_0000_0000_0000_0000_1000_0000_0000;
  }
}

pub enum Colour
{
  Default,
  RGB       ( u8, u8, u8 ),             // ( r, g, b ): 0–255, 0–255, 0–255
  Standard  ( u8 ),                     // console: 0–7
  Bright    ( u8 ),                     // console: 0–7
  Cube      ( u8, u8, u8 ),             // ( r, g, b ): 0–5, 0–5, 0–5
  Grey      ( u8 ),                     // console: 0–23
  Black,                                // #000000
  Red,                                  // #ff0000
  Green,                                // #00ff00
  Brown,
  Blue,                                 // #0000ff
  Purple,                               // #ff00ff
  Cyan,                                 // #00ffff
  LightGrey,
  DarkGrey,
  LightRed,
  LightGreen,
  Yellow,                               // #ffff00
  LightBlue,
  LightPurple,
  LightCyan,
  White,                                // #ffffff
  BrightBlack,
  BrightRed,
  BrightGreen,
  BrightYellow,
  BrightBlue,
  BrightPurple,
  BrightCyan,
  BrightWhite,
  FaintBlack,
  FaintRed,
  FaintGreen,
  FaintYellow,
  FaintBlue,
  FaintPurple,
  FaintCyan,
  FaintWhite,
}

pub struct StyledToken
{
  pub word:                             String,
  pub flags:                            Style,
  pub font:                             u8,
  pub fgColour:                         Colour,
  pub bgColour:                         Colour,
}

impl StyledToken
{
  pub fn new
  (
    word:                               String,
    flags:                              Style,
    font:                               u8,
    fgColour:                           Colour,
    bgColour:                           Colour,
  ) -> Self
  {
    Self
    {
      word:                             word,
      flags:                            flags,
      font:                             font,
      fgColour:                         fgColour,
      bgColour:                         bgColour,
    }
  }
}
