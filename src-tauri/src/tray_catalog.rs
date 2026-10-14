// Exact upstream 237-pattern structure for System Tray
pub struct TrayCategory {
    pub title: &'static str,
    pub subgroups: &'static [TraySubgroup],
    pub styles: &'static [(&'static str, &'static str)],
}

pub struct TraySubgroup {
    pub title: &'static str,
    pub styles: &'static [(&'static str, &'static str)],
}

pub static ROOT_STYLES: &[(&str, &str)] = &[
    ("tabriz", "Tabriz After Midnight"),
    ("klasik", "Madder Red"),
    ("lacivert", "Midnight"),
    ("zumrut", "Emerald"),
    ("kilim", "Kilim"),
    ("gorunmez", "Invisible"),
];

static STYLES_SUB_1: &[(&str, &str)] = &[
    ("selbu", "Selbu Rose"),
    ("rosemaling", "Rosemaling"),
    ("hardanger", "Hardanger Embroidery"),
    ("setesdal", "Setesdal Sweater"),
    ("somon", "Salmon"),
    ("solje", "Sølje Brooches"),
    ("fiyort", "The Fjord"),
    ("lofoten", "Lofoten"),
    ("bryggen", "Bryggen"),
    ("vikinggemisi", "Viking Ship"),
    ("vafler", "Heart Waffles"),
    ("norvecart", "Greetings from Norway"),
];

static STYLES_SUB_2: &[(&str, &str)] = &[
    ("dalahast", "Dala Horses"),
    ("kurbits", "Kurbits"),
    ("lingon", "Lingonberries"),
    ("midsommar", "Midsummer Wreath"),
    ("skane", "Skåne Embroidery"),
    ("kraftor", "Crayfish"),
    ("stuga", "Red Cottage"),
    ("majstang", "Midsummer"),
    ("skargard", "The Archipelago"),
    ("gamlastan", "Gamla Stan"),
    ("fika", "Fika"),
    ("isveckart", "Greetings from Sweden"),
];

static STYLES_SUB_3: &[(&str, &str)] = &[
    ("kalpler", "Woven Hearts"),
    ("dugum", "Sailor’s Knots"),
    ("moderne", "Danish Modern"),
    ("sild", "Herring"),
    ("kehribar", "Amber"),
    ("kugu", "Swans"),
    ("nyhavn", "Nyhavn"),
    ("smorrebrod", "Smørrebrød"),
    ("wienerbrod", "Danish Bakery"),
    ("bisiklet", "Copenhagen Bikes"),
    ("monsklint", "Møns Klint"),
    ("danimarkakart", "Greetings from Denmark"),
];

static STYLES_SUB_4: &[(&str, &str)] = &[
    ("ryijy", "Ryijy"),
    ("himmeli", "Himmeli"),
    ("tuohi", "Birch Bark"),
    ("lakka", "Cloudberries"),
    ("karjala", "Karelian Embroidery"),
    ("lumikide", "Snow Crystals"),
    ("binjarvi", "A Thousand Lakes"),
    ("sauna", "Lakeside Sauna"),
    ("laponya", "Lapland"),
    ("tramvay", "Helsinki Tram"),
    ("piirakka", "Karelian Pies"),
    ("finlandiyakart", "Greetings from Finland"),
];

static STYLES_SUB_5: &[(&str, &str)] = &[
    ("lopapeysa", "Lopapeysa"),
    ("bazalt", "Basalt Columns"),
    ("lundi", "Puffins"),
    ("koyun", "Icelandic Sheep"),
    ("lav", "Lava and Moss"),
    ("laufabraud", "Leaf Bread"),
    ("gayzer", "The Geyser"),
    ("skogafoss", "Skógafoss"),
    ("karakum", "Black Sand Beach"),
    ("jokulsarlon", "Glacier Lagoon"),
    ("reykjavik", "Reykjavík Roofs"),
    ("izlandakart", "Greetings from Iceland"),
];

static SUBGROUPS_6: &[TraySubgroup] = &[
    TraySubgroup { title: "Norway", styles: STYLES_SUB_1 },
    TraySubgroup { title: "Sweden", styles: STYLES_SUB_2 },
    TraySubgroup { title: "Denmark", styles: STYLES_SUB_3 },
    TraySubgroup { title: "Finland", styles: STYLES_SUB_4 },
    TraySubgroup { title: "Iceland", styles: STYLES_SUB_5 },
];

static STYLES_SUB_7: &[(&str, &str)] = &[
    ("majolika", "Majolica"),
    ("limon", "Amalfi Lemons"),
    ("damask", "Venetian Damask"),
    ("mozaik", "Roman Mosaic"),
    ("floransa", "Florentine Marbling"),
    ("murano", "Murano Glass"),
    ("makarna", "Pasta"),
    ("sicilya", "Sicilian Cart"),
    ("venedik", "Venice"),
    ("toskana", "Tuscany"),
    ("cinqueterre", "Cinque Terre"),
    ("pizza", "Pizza"),
    ("italyakart", "Greetings from Italy"),
    ("kolezyum", "The Colosseum"),
    ("gelato", "Gelato"),
    ("romasokak", "Roman Street"),
];

static STYLES_SUB_8: &[(&str, &str)] = &[
    ("menderes", "Meander"),
    ("amfora", "Amphora"),
    ("zeytin", "Olive Branch"),
    ("yunuslar", "Dolphins"),
    ("bizans", "Byzantine Mosaic"),
    ("ahtapot", "Minoan Octopus"),
    ("skiros", "Skyros Embroidery"),
    ("kirlangic", "Swallow Fresco"),
    ("santorini", "Santorini"),
    ("akropolis", "The Acropolis"),
    ("liman", "Fishing Harbor"),
    ("mikonos", "Mykonos"),
    ("yunankart", "Greetings from Greece"),
    ("knossos", "Knossos"),
    ("mavikapi", "The Blue Door"),
    ("yunansalata", "Greek Salad"),
];

static STYLES_SUB_9: &[(&str, &str)] = &[
    ("azulejo", "Azulejo"),
    ("flamenko", "Flamenco"),
    ("manton", "Manila Shawl"),
    ("granada", "Pomegranates"),
    ("trencadis", "Trencadís"),
    ("dantel", "Mantilla Lace"),
    ("gitar", "Spanish Guitar"),
    ("kastanyet", "Castanets"),
    ("elhamra", "The Alhambra"),
    ("avlu", "Andalusian Patio"),
    ("paella", "Paella"),
    ("lamancha", "La Mancha"),
    ("ispanyakart", "Greetings from Spain"),
    ("parkguell", "Park Güell"),
    ("tapas", "Tapas"),
    ("portakal", "Valencia Oranges"),
];

static STYLES_SUB_10: &[(&str, &str)] = &[
    ("provence", "Provençal Print"),
    ("tomette", "Tomette Tiles"),
    ("cigale", "Cicadas"),
    ("mimoza", "Mimosa"),
    ("toile", "Toile"),
    ("moustiers", "Moustiers Faience"),
    ("lavanta", "Lavender Fields"),
    ("riviera", "Riviera"),
    ("aycicegi", "Sunflowers"),
    ("petank", "Pétanque"),
    ("pazar", "Provence Market"),
    ("kalanka", "Calanques"),
    ("fransakart", "Greetings from France"),
    ("mavisandalye", "Blue Chairs"),
    ("roussillon", "Roussillon"),
    ("camargue", "Camargue Horses"),
];

static SUBGROUPS_11: &[TraySubgroup] = &[
    TraySubgroup { title: "Italy", styles: STYLES_SUB_7 },
    TraySubgroup { title: "Greece", styles: STYLES_SUB_8 },
    TraySubgroup { title: "Spain", styles: STYLES_SUB_9 },
    TraySubgroup { title: "France", styles: STYLES_SUB_10 },
];

static STYLES_CAT_12: &[(&str, &str)] = &[
    ("balkabagi", "Pumpkin Patch"),
    ("orumcek", "Spider Web"),
    ("hayalet", "Ghost Night"),
];

static STYLES_SUB_13: &[(&str, &str)] = &[
    ("yildizlar", "Stars & Stripes"),
    ("mesa", "Desert Mesa"),
    ("yorgan", "Log Cabin Quilt"),
    ("ekose", "Buffalo Plaid"),
    ("kanyon", "Canyon Sunset"),
    ("ambar", "Barn Star"),
    ("yalniz", "Lone Star"),
    ("bizon", "Bison Plains"),
    ("kartpostal", "Greetings from America"),
    ("millipark", "See America"),
    ("beyzbol", "Ballpark"),
    ("basketbol", "Hardwood Court"),
    ("diner", "Diner"),
    ("route66", "Route 66"),
];

static STYLES_SUB_14: &[(&str, &str)] = &[
    ("akcaagac", "Maple Leaf"),
    ("yaprak", "Maple Leaf Quilt"),
    ("kayalik", "Rocky Mountains"),
    ("atlantik", "Atlantic Tartan"),
    ("geyik", "Moose Lake"),
    ("kazlar", "Canada Geese"),
    ("dalgic", "Loon at Dawn"),
    ("kanadakart", "Greetings from Canada"),
    ("hokey", "Hockey Rink"),
    ("kuzeyisik", "Northern Lights"),
    ("fener", "Lighthouse"),
    ("kutupayisi", "Polar Bear"),
];

static STYLES_SUB_15: &[(&str, &str)] = &[
    ("sarape", "Sarape"),
    ("greca", "Greca"),
    ("cini", "Blue Tile"),
    ("agave", "Agave Fields"),
    ("kafatasi", "Sugar Skull"),
    ("monark", "Monarch Butterflies"),
    ("jaguar", "Painted Jaguar"),
    ("meksikakart", "Greetings from Mexico"),
    ("piramit", "Pyramid"),
    ("sokak", "Colorful Street"),
    ("trajinera", "Trajinera"),
    ("tulum", "Tulum Beach"),
];

static SUBGROUPS_16: &[TraySubgroup] = &[
    TraySubgroup { title: "USA", styles: STYLES_SUB_13 },
    TraySubgroup { title: "Canada", styles: STYLES_SUB_14 },
    TraySubgroup { title: "Mexico", styles: STYLES_SUB_15 },
];

static STYLES_CAT_17: &[(&str, &str)] = &[
    ("heriz", "Heriz"),
    ("bahce", "Paradise Garden"),
    ("av", "Hunting Scene"),
    ("hayat", "Tree of Life"),
    ("buta", "Boteh"),
    ("bahtiyari", "Bakhtiari"),
    ("gabbe", "Qashqai Gabbeh"),
    ("nain", "Nain"),
    ("vazo", "Kerman Vase"),
    ("herati", "Herati"),
    ("kum", "Qom Silk"),
    ("turkmen", "Turkmen Gul"),
];

static STYLES_CAT_18: &[(&str, &str)] = &[
    ("usak", "Ushak Medallion"),
    ("lotto", "Lotto"),
    ("bergama", "Bergama"),
    ("yagcibedir", "Yağcıbedir"),
    ("selcuklu", "Seljuk"),
    ("hereke", "Hereke Silk"),
    ("iznik", "Iznik Tulips"),
    ("turkiyekart", "Greetings from Türkiye"),
    ("kapadokya", "Cappadocia Balloons"),
    ("bogaz", "Bosphorus Ferry"),
    ("pamukkale", "Pamukkale"),
    ("nazar", "Nazar Bead"),
];

static STYLES_SUB_19: &[(&str, &str)] = &[
    ("bojagi", "Bojagi"),
    ("saekdong", "Saekdong Stripes"),
    ("seladon", "Celadon Cranes"),
    ("dancheong", "Dancheong"),
    ("hangeul", "Hangeul"),
    ("najeon", "Mother-of-Pearl"),
    ("sipjang", "Ten Longevities"),
    ("kaplan", "Magpie and Tiger"),
    ("hanok", "Hanok"),
    ("seul", "Seoul at Night"),
    ("korekart", "Greetings from Korea"),
    ("yeondeung", "Lotus Lanterns"),
    ("bibimbap", "Bibimbap"),
    ("jeju", "Jeju Island"),
];

static STYLES_SUB_20: &[(&str, &str)] = &[
    ("seigaiha", "Seigaiha Waves"),
    ("asanoha", "Asanoha"),
    ("boro", "Boro"),
    ("koi", "Koi Pond"),
    ("byobu", "Gold Screen"),
    ("shibori", "Shibori"),
    ("zen", "Zen Garden"),
    ("fuji", "Fuji and Sakura"),
    ("kyoto", "Kyoto Lane"),
    ("japonkart", "Greetings from Japan"),
    ("buyukdalga", "The Great Wave"),
    ("manekineko", "Maneki-neko"),
    ("torii", "Torii Path"),
    ("sushi", "Sushi"),
];

static STYLES_SUB_21: &[(&str, &str)] = &[
    ("porselen", "Blue and White"),
    ("bulut", "Lucky Clouds"),
    ("artdeco", "Shanghai Deco"),
    ("ningxia", "Ningxia Peony"),
    ("bambu", "Ink Bamboo"),
    ("kagitkesim", "Paper-Cut"),
    ("imparator", "Imperial Waves"),
    ("mahjong", "Mahjong"),
    ("panda", "Panda"),
    ("seddicin", "The Great Wall"),
    ("cinkart", "Greetings from China"),
    ("fenerfest", "Lantern Festival"),
    ("guilin", "Guilin"),
    ("dimsum", "Dim Sum"),
];

static SUBGROUPS_22: &[TraySubgroup] = &[
    TraySubgroup { title: "Korea", styles: STYLES_SUB_19 },
    TraySubgroup { title: "Japan", styles: STYLES_SUB_20 },
    TraySubgroup { title: "China", styles: STYLES_SUB_21 },
];

pub static TRAY_CATEGORIES: &[TrayCategory] = &[
    TrayCategory { title: "Nordic · NEW", subgroups: SUBGROUPS_6, styles: &[] },
    TrayCategory { title: "Mediterranean", subgroups: SUBGROUPS_11, styles: &[] },
    TrayCategory { title: "Halloween", subgroups: &[], styles: STYLES_CAT_12 },
    TrayCategory { title: "Americas", subgroups: SUBGROUPS_16, styles: &[] },
    TrayCategory { title: "Persia", subgroups: &[], styles: STYLES_CAT_17 },
    TrayCategory { title: "Türkiye", subgroups: &[], styles: STYLES_CAT_18 },
    TrayCategory { title: "East Asia", subgroups: SUBGROUPS_22, styles: &[] },
];
