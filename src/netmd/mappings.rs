use phf::phf_map;

pub static MAPPINGS_JP: phf::Map<&'static str, &'static str> = phf_map!{
        "!" =>"！",
        "\"" =>"＂",
        "#" =>"＃",
        "$" =>"＄",
        "%" =>"％",
        "&" =>"＆",
        "'" =>"＇",
        "" =>"（",
        ")" =>"）",
        "*" =>"＊",
        "+" =>"＋",
        "," =>"，",
        "-" =>"－",
        "." =>"．",
        "/" =>"／",
        ":" =>"：",
        ";" =>"；",
        "<" =>"＜",
        "=" =>"＝",
        ">" =>"＞",
        "?" =>"？",
        "@" =>"＠",
        "A" =>"Ａ",
        "B" =>"Ｂ",
        "C" =>"Ｃ",
        "D" =>"Ｄ",
        "E" =>"Ｅ",
        "F" =>"Ｆ",
        "G" =>"Ｇ",
        "H" =>"Ｈ",
        "I" =>"Ｉ",
        "J" =>"Ｊ",
        "K" =>"Ｋ",
        "L" =>"Ｌ",
        "M" =>"Ｍ",
        "N" =>"Ｎ",
        "O" =>"Ｏ",
        "P" =>"Ｐ",
        "Q" =>"Ｑ",
        "R" =>"Ｒ",
        "S" =>"Ｓ",
        "T" =>"Ｔ",
        "U" =>"Ｕ",
        "V" =>"Ｖ",
        "W" =>"Ｗ",
        "X" =>"Ｘ",
        "Y" =>"Ｙ",
        "Z" =>"Ｚ",
        "[" =>"［",
        "\\" =>"＼",
        "]" =>"］",
        "^" =>"＾",
        "_" =>"＿",
        "`" =>"｀",
        "a" =>"ａ",
        "b" =>"ｂ",
        "c" =>"ｃ",
        "d" =>"ｄ",
        "e" =>"ｅ",
        "f" =>"ｆ",
        "g" =>"ｇ",
        "h" =>"ｈ",
        "i" =>"ｉ",
        "j" =>"ｊ",
        "k" =>"ｋ",
        "l" =>"ｌ",
        "m" =>"ｍ",
        "n" =>"ｎ",
        "o" =>"ｏ",
        "p" =>"ｐ",
        "q" =>"ｑ",
        "r" =>"ｒ",
        "s" =>"ｓ",
        "t" =>"ｔ",
        "u" =>"ｕ",
        "v" =>"ｖ",
        "w" =>"ｗ",
        "x" =>"ｘ",
        "y" =>"ｙ",
        "z" =>"ｚ",
        "{" =>"｛",
        "|" =>"｜",
        "}" =>"｝",
        "~" =>"～",
        " " =>"\u{3000}",
        "0" =>"０",
        "1" =>"１",
        "2" =>"２",
        "3" =>"３",
        "4" =>"４",
        "5" =>"５",
        "6" =>"６",
        "7" =>"７",
        "8" =>"８",
        "9" =>"９",
        "ｧ" =>"ァ",
        "ｱ" =>"ア",
        "ｨ" =>"ィ",
        "ｲ" =>"イ",
        "ｩ" =>"ゥ",
        "ｳ" =>"ウ",
        "ｪ" =>"ェ",
        "ｴ" =>"エ",
        "ｫ" =>"ォ",
        "ｵ" =>"オ",
        "ｶ" =>"カ",
        "ｶﾞ" =>"ガ",
        "ｷ" =>"キ",
        "ｷﾞ" =>"ギ",
        "ｸ" =>"ク",
        "ｸﾞ" =>"グ",
        "ｹ" =>"ケ",
        "ｹﾞ" =>"ゲ",
        "ｺ" =>"コ",
        "ｺﾞ" =>"ゴ",
        "ｻ" =>"サ",
        "ｻﾞ" =>"ザ",
        "ｼ" =>"シ",
        "ｼﾞ" =>"ジ",
        "ｽ" =>"ス",
        "ｽﾞ" =>"ズ",
        "ｾ" =>"セ",
        "ｾﾞ" =>"ゼ",
        "ｿ" =>"ソ",
        "ｿﾞ" =>"ゾ",
        "ﾀ" =>"タ",
        "ﾀﾞ" =>"ダ",
        "ﾁ" =>"チ",
        "ﾁﾞ" =>"ヂ",
        "ｯ" =>"ッ",
        "ﾂ" =>"ツ",
        "ﾂﾞ" =>"ヅ",
        "ﾃ" =>"テ",
        "ﾃﾞ" =>"デ",
        "ﾄ" =>"ト",
        "ﾄﾞ" =>"ド",
        "ﾅ" =>"ナ",
        "ﾆ" =>"ニ",
        "ﾇ" =>"ヌ",
        "ﾈ" =>"ネ",
        "ﾉ" =>"ノ",
        "ﾊ" =>"ハ",
        "ﾊﾞ" =>"バ",
        "ﾊﾟ" =>"パ",
        "ﾋ" =>"ヒ",
        "ﾋﾞ" =>"ビ",
        "ﾋﾟ" =>"ピ",
        "ﾌ" =>"フ",
        "ﾌﾞ" =>"ブ",
        "ﾌﾟ" =>"プ",
        "ﾍ" =>"ヘ",
        "ﾍﾞ" =>"ベ",
        "ﾍﾟ" =>"ペ",
        "ﾎ" =>"ホ",
        "ﾎﾞ" =>"ボ",
        "ﾎﾟ" =>"ポ",
        "ﾏ" =>"マ",
        "ﾐ" =>"ミ",
        "ﾑ" =>"ム",
        "ﾒ" =>"メ",
        "ﾓ" =>"モ",
        "ｬ" =>"ャ",
        "ﾔ" =>"ヤ",
        "ｭ" =>"ュ",
        "ﾕ" =>"ユ",
        "ｮ" =>"ョ",
        "ﾖ" =>"ヨ",
        "ﾗ" =>"ラ",
        "ﾘ" =>"リ",
        "ﾙ" =>"ル",
        "ﾚ" =>"レ",
        "ﾛ" =>"ロ",
        "ﾜ" =>"ワ",
        "ｦ" =>"ヲ",
        "ﾝ" =>"ン",
        "ｰ" =>"ー",
        "ヮ" =>"ヮ",
        "ヰ" =>"ヰ",
        "ヱ" =>"ヱ",
        "ヵ" =>"ヵ",
        "ヶ" =>"ヶ",
        "ｳﾞ" =>"ヴ",
        "ヽ" =>"ヽ",
        "ヾ" =>"ヾ",
        "･" =>"・",
        "｢" =>"「",
        "｣" =>"」",
        "｡" =>"。",
        "､" =>"、"
};
pub static MAPPINGS_RU: phf::Map<&'static str, &'static str> = phf_map!{
        "а" =>"a",
        "б" =>"b",
        "в" =>"v",
        "г" =>"g",
        "д" =>"d",
        "е" =>"e",
        "ё" =>"e",
        "ж" =>"zh",
        "з" =>"z",
        "и" =>"i",
        "й" =>"i",
        "к" =>"k",
        "л" =>"l",
        "м" =>"m",
        "н" =>"n",
        "о" =>"o",
        "п" =>"p",
        "р" =>"r",
        "с" =>"s",
        "т" =>"t",
        "у" =>"u",
        "ф" =>"f",
        "х" =>"kh",
        "ц" =>"tc",
        "ч" =>"ch",
        "ш" =>"sh",
        "щ" =>"shch",
        "ъ" =>"",
        "ы" =>"y",
        "ь" =>"'",
        "э" =>"e",
        "ю" =>"iu",
        "я" =>"ia",
        "А" =>"A",
        "Б" =>"B",
        "В" =>"V",
        "Г" =>"G",
        "Д" =>"D",
        "Е" =>"E",
        "Ё" =>"E",
        "Ж" =>"Zh",
        "З" =>"Z",
        "И" =>"I",
        "Й" =>"I",
        "К" =>"K",
        "Л" =>"L",
        "М" =>"M",
        "Н" =>"N",
        "О" =>"O",
        "П" =>"P",
        "Р" =>"R",
        "С" =>"S",
        "Т" =>"T",
        "У" =>"U",
        "Ф" =>"F",
        "Х" =>"Kh",
        "Ц" =>"Tc",
        "Ч" =>"Ch",
        "Ш" =>"Sh",
        "Щ" =>"Shch",
        "Ъ" =>"",
        "Ы" =>"Y",
        "Ь" =>"'",
        "Э" =>"E",
        "Ю" =>"Iu",
        "Я" =>"Ia"
};
pub static MAPPINGS_DE: phf::Map<&'static str, &'static str> = phf_map!{
    "Ä" => "Ae",
    "ä" => "ae",
    "Ö" => "Oe",
    "ö" => "oe",
    "Ü" => "Ue",
    "ü" => "ue",
    "ß" => "ss"
};
pub static MAPPINGS_HW: phf::Map<&'static str, &'static str> = phf_map!{
        "－" =>"-",
        "ｰ" =>"-",
        "ァ" =>"ｧ",
        "ア" =>"ｱ",
        "ィ" =>"ｨ",
        "イ" =>"ｲ",
        "ゥ" =>"ｩ",
        "ウ" =>"ｳ",
        "ェ" =>"ｪ",
        "エ" =>"ｴ",
        "ォ" =>"ｫ",
        "オ" =>"ｵ",
        "カ" =>"ｶ",
        "ガ" =>"ｶﾞ",
        "キ" =>"ｷ",
        "ギ" =>"ｷﾞ",
        "ク" =>"ｸ",
        "グ" =>"ｸﾞ",
        "ケ" =>"ｹ",
        "ゲ" =>"ｹﾞ",
        "コ" =>"ｺ",
        "ゴ" =>"ｺﾞ",
        "サ" =>"ｻ",
        "ザ" =>"ｻﾞ",
        "シ" =>"ｼ",
        "ジ" =>"ｼﾞ",
        "ス" =>"ｽ",
        "ズ" =>"ｽﾞ",
        "セ" =>"ｾ",
        "ゼ" =>"ｾﾞ",
        "ソ" =>"ｿ",
        "ゾ" =>"ｿﾞ",
        "タ" =>"ﾀ",
        "ダ" =>"ﾀﾞ",
        "チ" =>"ﾁ",
        "ヂ" =>"ﾁﾞ",
        "ッ" =>"ｯ",
        "ツ" =>"ﾂ",
        "ヅ" =>"ﾂﾞ",
        "テ" =>"ﾃ",
        "デ" =>"ﾃﾞ",
        "ト" =>"ﾄ",
        "ド" =>"ﾄﾞ",
        "ナ" =>"ﾅ",
        "ニ" =>"ﾆ",
        "ヌ" =>"ﾇ",
        "ネ" =>"ﾈ",
        "ノ" =>"ﾉ",
        "ハ" =>"ﾊ",
        "バ" =>"ﾊﾞ",
        "パ" =>"ﾊﾟ",
        "ヒ" =>"ﾋ",
        "ビ" =>"ﾋﾞ",
        "ピ" =>"ﾋﾟ",
        "フ" =>"ﾌ",
        "ブ" =>"ﾌﾞ",
        "プ" =>"ﾌﾟ",
        "ヘ" =>"ﾍ",
        "ベ" =>"ﾍﾞ",
        "ペ" =>"ﾍﾟ",
        "ホ" =>"ﾎ",
        "ボ" =>"ﾎﾞ",
        "ポ" =>"ﾎﾟ",
        "マ" =>"ﾏ",
        "ミ" =>"ﾐ",
        "ム" =>"ﾑ",
        "メ" =>"ﾒ",
        "モ" =>"ﾓ",
        "ャ" =>"ｬ",
        "ヤ" =>"ﾔ",
        "ュ" =>"ｭ",
        "ユ" =>"ﾕ",
        "ョ" =>"ｮ",
        "ヨ" =>"ﾖ",
        "ラ" =>"ﾗ",
        "リ" =>"ﾘ",
        "ル" =>"ﾙ",
        "レ" =>"ﾚ",
        "ロ" =>"ﾛ",
        "ワ" =>"ﾜ",
        "ヲ" =>"ｦ",
        "ン" =>"ﾝ",
        "ー" =>"-",
        "ヮ" =>"ヮ",
        "ヰ" =>"ヰ",
        "ヱ" =>"ヱ",
        "ヵ" =>"ヵ",
        "ヶ" =>"ヶ",
        "ヴ" =>"ｳﾞ",
        "ヽ" =>"ヽ",
        "ヾ" =>"ヾ",
        "・" =>"･",
        "「" =>"｢",
        "」" =>"｣",
        "。" =>"｡",
        "、" =>"､",
        "！" =>"!",
        "＂" =>"\"",
        "＃" =>"#",
        "＄" =>"$",
        "％" =>"%",
        "＆" =>"&",
        "＇" =>"'",
        "（" =>"",
        "）" =>")",
        "＊" =>"*",
        "＋" =>"+",
        "，" =>",",
        "．" =>".",
        "／" =>"/",
        "：" =>":",
        "；" =>";",
        "＜" =>"<",
        "＝" =>"=",
        "＞" =>">",
        "？" =>"?",
        "＠" =>"@",
        "Ａ" =>"A",
        "Ｂ" =>"B",
        "Ｃ" =>"C",
        "Ｄ" =>"D",
        "Ｅ" =>"E",
        "Ｆ" =>"F",
        "Ｇ" =>"G",
        "Ｈ" =>"H",
        "Ｉ" =>"I",
        "Ｊ" =>"J",
        "Ｋ" =>"K",
        "Ｌ" =>"L",
        "Ｍ" =>"M",
        "Ｎ" =>"N",
        "Ｏ" =>"O",
        "Ｐ" =>"P",
        "Ｑ" =>"Q",
        "Ｒ" =>"R",
        "Ｓ" =>"S",
        "Ｔ" =>"T",
        "Ｕ" =>"U",
        "Ｖ" =>"V",
        "Ｗ" =>"W",
        "Ｘ" =>"X",
        "Ｙ" =>"Y",
        "Ｚ" =>"Z",
        "［" =>"[",
        "＼" =>"\\",
        "］" =>"]",
        "＾" =>"^",
        "＿" =>"_",
        "｀" =>"`",
        "ａ" =>"a",
        "ｂ" =>"b",
        "ｃ" =>"c",
        "ｄ" =>"d",
        "ｅ" =>"e",
        "ｆ" =>"f",
        "ｇ" =>"g",
        "ｈ" =>"h",
        "ｉ" =>"i",
        "ｊ" =>"j",
        "ｋ" =>"k",
        "ｌ" =>"l",
        "ｍ" =>"m",
        "ｎ" =>"n",
        "ｏ" =>"o",
        "ｐ" =>"p",
        "ｑ" =>"q",
        "ｒ" =>"r",
        "ｓ" =>"s",
        "ｔ" =>"t",
        "ｕ" =>"u",
        "ｖ" =>"v",
        "ｗ" =>"w",
        "ｘ" =>"x",
        "ｙ" =>"y",
        "ｚ" =>"z",
        "｛" =>"{",
        "｜" =>"|",
        "｝" =>"}",
        "～" =>"~",
        "　" =>" ",
        "０" =>"0",
        "１" =>"1",
        "２" =>"2",
        "３" =>"3",
        "４" =>"4",
        "５" =>"5",
        "６" =>"6",
        "７" =>"7",
        "８" =>"8",
        "９" =>"9",
        "ぁ" =>"ｧ",
        "あ" =>"ｱ",
        "ぃ" =>"ｨ",
        "い" =>"ｲ",
        "ぅ" =>"ｩ",
        "う" =>"ｳ",
        "ぇ" =>"ｪ",
        "え" =>"ｴ",
        "ぉ" =>"ｫ",
        "お" =>"ｵ",
        "か" =>"ｶ",
        "が" =>"ｶﾞ",
        "き" =>"ｷ",
        "ぎ" =>"ｷﾞ",
        "く" =>"ｸ",
        "ぐ" =>"ｸﾞ",
        "け" =>"ｹ",
        "げ" =>"ｹﾞ",
        "こ" =>"ｺ",
        "ご" =>"ｺﾞ",
        "さ" =>"ｻ",
        "ざ" =>"ｻﾞ",
        "し" =>"ｼ",
        "じ" =>"ｼﾞ",
        "す" =>"ｽ",
        "ず" =>"ｽﾞ",
        "せ" =>"ｾ",
        "ぜ" =>"ｾﾞ",
        "そ" =>"ｿ",
        "ぞ" =>"ｿﾞ",
        "た" =>"ﾀ",
        "だ" =>"ﾀﾞ",
        "ち" =>"ﾁ",
        "ぢ" =>"ﾁﾞ",
        "っ" =>"ｯ",
        "つ" =>"ﾂ",
        "づ" =>"ﾂﾞ",
        "て" =>"ﾃ",
        "で" =>"ﾃﾞ",
        "と" =>"ﾄ",
        "ど" =>"ﾄﾞ",
        "な" =>"ﾅ",
        "に" =>"ﾆ",
        "ぬ" =>"ﾇ",
        "ね" =>"ﾈ",
        "の" =>"ﾉ",
        "は" =>"ﾊ",
        "ば" =>"ﾊﾞ",
        "ぱ" =>"ﾊﾟ",
        "ひ" =>"ﾋ",
        "び" =>"ﾋﾞ",
        "ぴ" =>"ﾋﾟ",
        "ふ" =>"ﾌ",
        "ぶ" =>"ﾌﾞ",
        "ぷ" =>"ﾌﾟ",
        "へ" =>"ﾍ",
        "べ" =>"ﾍﾞ",
        "ぺ" =>"ﾍﾟ",
        "ほ" =>"ﾎ",
        "ぼ" =>"ﾎﾞ",
        "ぽ" =>"ﾎﾟ",
        "ま" =>"ﾏ",
        "み" =>"ﾐ",
        "む" =>"ﾑ",
        "め" =>"ﾒ",
        "も" =>"ﾓ",
        "ゃ" =>"ｬ",
        "や" =>"ﾔ",
        "ゅ" =>"ｭ",
        "ゆ" =>"ﾕ",
        "ょ" =>"ｮ",
        "よ" =>"ﾖ",
        "ら" =>"ﾗ",
        "り" =>"ﾘ",
        "る" =>"ﾙ",
        "れ" =>"ﾚ",
        "ろ" =>"ﾛ",
        "わ" =>"ﾜ",
        "を" =>"ｦ",
        "ん" =>"ﾝ",
        "ゎ" =>"ヮ",
        "ゐ" =>"ヰ",
        "ゑ" =>"ヱ",
        "ゕ" =>"ヵ",
        "ゖ" =>"ヶ",
        "ゔ" =>"ｳﾞ",
        "ゝ" =>"ヽ",
        "ゞ" =>"ヾ",
};
pub static ALLOWED_HW_KANA: &[&'static str] = &[
        "-",
        "-",
        "ｧ",
        "ｱ",
        "ｨ",
        "ｲ",
        "ｩ",
        "ｳ",
        "ｪ",
        "ｴ",
        "ｫ",
        "ｵ",
        "ｶ",
        "ｶﾞ",
        "ｷ",
        "ｷﾞ",
        "ｸ",
        "ｸﾞ",
        "ｹ",
        "ｹﾞ",
        "ｺ",
        "ｺﾞ",
        "ｻ",
        "ｻﾞ",
        "ｼ",
        "ｼﾞ",
        "ｽ",
        "ｽﾞ",
        "ｾ",
        "ｾﾞ",
        "ｿ",
        "ｿﾞ",
        "ﾀ",
        "ﾀﾞ",
        "ﾁ",
        "ﾁﾞ",
        "ｯ",
        "ﾂ",
        "ﾂﾞ",
        "ﾃ",
        "ﾃﾞ",
        "ﾄ",
        "ﾄﾞ",
        "ﾅ",
        "ﾆ",
        "ﾇ",
        "ﾈ",
        "ﾉ",
        "ﾊ",
        "ﾊﾞ",
        "ﾊﾟ",
        "ﾋ",
        "ﾋﾞ",
        "ﾋﾟ",
        "ﾌ",
        "ﾌﾞ",
        "ﾌﾟ",
        "ﾍ",
        "ﾍﾞ",
        "ﾍﾟ",
        "ﾎ",
        "ﾎﾞ",
        "ﾎﾟ",
        "ﾏ",
        "ﾐ",
        "ﾑ",
        "ﾒ",
        "ﾓ",
        "ｬ",
        "ﾔ",
        "ｭ",
        "ﾕ",
        "ｮ",
        "ﾖ",
        "ﾗ",
        "ﾘ",
        "ﾙ",
        "ﾚ",
        "ﾛ",
        "ﾜ",
        "ｦ",
        "ﾝ",
        "-",
        "ヮ",
        "ヰ",
        "ヱ",
        "ヵ",
        "ヶ",
        "ｳﾞ",
        "ヽ",
        "ヾ",
        "･",
        "｢",
        "｣",
        "｡",
        "､",
        "!",
        "\"",
        "#",
        "$",
        "%",
        "&",
        "'",
        "",
        ")",
        "*",
        "+",
        ",",
        ".",
        "/",
        ":",
        ";",
        "<",
        "=",
        ">",
        "?",
        "@",
        "A",
        "B",
        "C",
        "D",
        "E",
        "F",
        "G",
        "H",
        "I",
        "J",
        "K",
        "L",
        "M",
        "N",
        "O",
        "P",
        "Q",
        "R",
        "S",
        "T",
        "U",
        "V",
        "W",
        "X",
        "Y",
        "Z",
        "[",
        "\\",
        "]",
        "^",
        "_",
        "`",
        "a",
        "b",
        "c",
        "d",
        "e",
        "f",
        "g",
        "h",
        "i",
        "j",
        "k",
        "l",
        "m",
        "n",
        "o",
        "p",
        "q",
        "r",
        "s",
        "t",
        "u",
        "v",
        "w",
        "x",
        "y",
        "z",
        "{",
        "|",
        "}",
        "~",
        " ",
        "0",
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        "ｧ",
        "ｱ",
        "ｨ",
        "ｲ",
        "ｩ",
        "ｳ",
        "ｪ",
        "ｴ",
        "ｫ",
        "ｵ",
        "ｶ",
        "ｶﾞ",
        "ｷ",
        "ｷﾞ",
        "ｸ",
        "ｸﾞ",
        "ｹ",
        "ｹﾞ",
        "ｺ",
        "ｺﾞ",
        "ｻ",
        "ｻﾞ",
        "ｼ",
        "ｼﾞ",
        "ｽ",
        "ｽﾞ",
        "ｾ",
        "ｾﾞ",
        "ｿ",
        "ｿﾞ",
        "ﾀ",
        "ﾀﾞ",
        "ﾁ",
        "ﾁﾞ",
        "ｯ",
        "ﾂ",
        "ﾂﾞ",
        "ﾃ",
        "ﾃﾞ",
        "ﾄ",
        "ﾄﾞ",
        "ﾅ",
        "ﾆ",
        "ﾇ",
        "ﾈ",
        "ﾉ",
        "ﾊ",
        "ﾊﾞ",
        "ﾊﾟ",
        "ﾋ",
        "ﾋﾞ",
        "ﾋﾟ",
        "ﾌ",
        "ﾌﾞ",
        "ﾌﾟ",
        "ﾍ",
        "ﾍﾞ",
        "ﾍﾟ",
        "ﾎ",
        "ﾎﾞ",
        "ﾎﾟ",
        "ﾏ",
        "ﾐ",
        "ﾑ",
        "ﾒ",
        "ﾓ",
        "ｬ",
        "ﾔ",
        "ｭ",
        "ﾕ",
        "ｮ",
        "ﾖ",
        "ﾗ",
        "ﾘ",
        "ﾙ",
        "ﾚ",
        "ﾛ",
        "ﾜ",
        "ｦ",
        "ﾝ",
        "ヮ",
        "ヰ",
        "ヱ",
        "ヵ",
        "ヶ",
        "ｳﾞ",
        "ヽ",
        "ヾ",
];
