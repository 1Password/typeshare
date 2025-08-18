package proto

import "encoding/json"

const MyIntVar uint32 = 12
const Empty string = ""
const SimpleAscii string = "Hello, world!"
const Multiline string = "Line1\nLine2\nLine3"
const EscapedCharacters string = "First\\line.\nSecond \"quoted\" line.\tEnd."
const Unicode string = "Emoji: 😄, Accented: café, Chinese: 世界"
const RawString string = `Raw \n, "quotes" are okay, and single \ is fine too`
const ContainsBacktick string = "Backtick: ` inside"
const ContainsDollarCurly string = "${not_interpolation}"
const EndsWithOddBackslash string = `Odd number of backslashes: \\\`
const NullByte string = "Null:\x00End"
const Combining string = "é vs é"
