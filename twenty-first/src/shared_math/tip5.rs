use std::ops::{Add, Mul};

use itertools::Itertools;
use num_bigint::BigInt;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};

use crate::shared_math::{
    ntt::ntt,
    traits::{Inverse, ModPowU32},
};

use super::{b_field_element::BFieldElement, polynomial::Polynomial, traits::PrimitiveRootOfUnity};

pub const DIGEST_LENGTH: usize = 5;
pub const STATE_SIZE: usize = 16;
pub const CAPACITY: usize = 6;
pub const RATE: usize = 10;
pub const NUM_ROUNDS: usize = 7;

pub const MDS: [u64; STATE_SIZE * STATE_SIZE] = [
    5910257123858819639,
    3449115226714951713,
    16770055338049327985,
    610399731775780810,
    7363016345531076300,
    16174724756564259629,
    8736587794472183152,
    12699016954477470956,
    13948112026909862966,
    18015813124076612987,
    9568929147539067610,
    14859461777592116402,
    18169364738825153183,
    18221568702798258352,
    1524268296724555606,
    5538821761600,
    1649528676200182784,
    336497118937017052,
    15805000027048028625,
    15709375513998678646,
    14837031240173858084,
    11366298206428370494,
    15698532768527519720,
    5911577595727321095,
    16676030327621016157,
    16537624251746851423,
    13325141695736654367,
    9337952653454313447,
    9090375522091353302,
    5605636660979522224,
    6357222834896114791,
    7776871531164456679,
    8264739868177574620,
    12732288338686680125,
    13022293791945187811,
    17403057736098613442,
    2871266924987061743,
    13286707530570640459,
    9229362695439112266,
    815317759014579856,
    7447771153889267897,
    2209002535000750347,
    3280506473249596174,
    13756142018694965622,
    10518080861296830621,
    16578355848983066277,
    12732532221704648123,
    3426526797578099186,
    8563516248221808333,
    13079317959606236131,
    15645458946300428515,
    9958819147895829140,
    13028053188247480206,
    6789511720078828478,
    6583246594815170294,
    4423695887326249884,
    9751139665897711642,
    10039202025292797758,
    12208726994829996150,
    6238795140281096003,
    9113696057226188857,
    9898705245385052191,
    4213712701625520075,
    8038355032286280912,
    426685147605824917,
    7673465577918025498,
    8452867379070564008,
    10827610229277395180,
    16155539332955658546,
    1575428636717115288,
    8765972548498757598,
    8405996249707890526,
    14855028677418679455,
    17878170012428694685,
    16572621079016066883,
    5311046098447994501,
    10635376800783355348,
    14205668690430323921,
    1181422971831412672,
    4651053123208915543,
    12465667489477238576,
    7300129031676503132,
    13458544786180633209,
    8946801771555977477,
    14203890406114400141,
    8219081892380458635,
    6035067543134909245,
    15140374581570897616,
    4514006299509426029,
    16757530089801321524,
    13202061911440346802,
    11227558237427129334,
    315998614524336401,
    11280705904396606227,
    5798516367202621128,
    17154761698338453414,
    13574436947400004837,
    3126509266905053998,
    10740979484255925394,
    9273322683773825324,
    15349096509718845737,
    14694022445619674948,
    8733857890739087596,
    3198488337424282101,
    9521016570828679381,
    11267736037298472148,
    14825280481028844943,
    1326588754335738002,
    6200834522767914499,
    1070210996042416038,
    9140190343656907671,
    15531381283521001952,
    253143295675927354,
    11977331414401291539,
    13941376566367813256,
    469904915148256197,
    10873951860155749104,
    3939719938926157877,
    2271392376641547055,
    4725974756185387075,
    14827835543640648161,
    17663273767033351157,
    12440960700789890843,
    16589620022628590428,
    12838889473653138505,
    11170336581460183657,
    7583333056198317221,
    6006908286410425140,
    15648567098514276013,
    188901633101859949,
    12256163716419861419,
    17319784688409668747,
    9648971065289440425,
    11370683735445551679,
    11265203235776280908,
    1737672785338087677,
    5225587291780939578,
    4739055740469849012,
    1212344601223444182,
    12958616893209019599,
    7922060480554370635,
    14661420107595710445,
    11744359917257111592,
    9674559564931202709,
    8326110231976411065,
    16856751238353701757,
    7515652322254196544,
    2062531989536141174,
    3875321171362100965,
    1164854003752487518,
    3997098993859160292,
    4074090397542250057,
    3050858158567944540,
    4568245569065883863,
    14559440781022773799,
    5401845794552358815,
    6544584366002554176,
    2511522072283652847,
    9759884967674698659,
    16411672358681189856,
    11392578809073737776,
    8013631514034873271,
    11439549174997471674,
    6373021446442411366,
    12491600135569477757,
    1017093281401495736,
    663547836518863091,
    16157302719777897692,
    11208801522915446640,
    10058178191286215107,
    5521712058210208094,
    3611681474253815005,
    4864578569041337696,
    12270319000993569289,
    7347066511426336318,
    6696546239958933736,
    3335469193383486908,
    12719366334180058014,
    14123019207894489639,
    11418186023060178542,
    2042199956854124583,
    17539253100488345226,
    16240833881391672847,
    11712520063241304909,
    6456900719511754234,
    1819022137223501306,
    7371152900053879920,
    6521878675261223812,
    2050999666988944811,
    8262038465464898064,
    13303819303390508091,
    12657292926928303663,
    8794128680724662595,
    4068577832515945116,
    758247715040138478,
    5600369601992438532,
    3369463178350382224,
    13763645328734311418,
    9685701761982837416,
    2711119809520557835,
    11680482056777716424,
    10958223503056770518,
    4168390070510137163,
    10823375744683484459,
    5613197991565754677,
    11781942063118564684,
    9352512500813609723,
    15997830646514778986,
    7407352006524266457,
    15312663387608602775,
    3026364159907661789,
    5698531403379362946,
    2544271242593770624,
    13104502948897878458,
    7840062700088318710,
    6028743588538970215,
    6144415809411296980,
    468368941216390216,
    3638618405705274008,
    11105401941482704573,
    1850274872877725129,
    1011155312563349004,
    3234620948537841909,
    3818372677739507813,
    4863130691592118581,
    8942166964590283171,
    3639677194051371072,
    15477372418124081864,
    10322228711752830209,
    9139111778956611066,
    202171733050704358,
    11982413146686512577,
    11001000478006340870,
    5491471715020327065,
    6969114856449768266,
    11088492421847219924,
    12913509272810999025,
    17366506887360149369,
    7036328554328346102,
    11139255730689011050,
    2844974929907956457,
    6488525141985913483,
    2860098796699131680,
    10366343151884073105,
    844875652557703984,
    1053177270393416978,
    5189466196833763142,
    1024738234713107670,
    8846741799369572841,
    14490406830213564822,
    10577371742628912722,
    3276210642025060502,
    2605621719516949928,
    5417148926702080639,
    11100652475866543814,
    5247366835775169839,
];

pub const MDS_INV: [u64; STATE_SIZE * STATE_SIZE] = [
    1572742562154761373,
    11904188991461183391,
    16702037635100780588,
    10395027733616703929,
    8130016957979279389,
    12091057987196709719,
    14570460902390750822,
    13452497170858892918,
    7302470671584418296,
    12930709087691977410,
    6940810864055149191,
    15479085069460687984,
    15273989414499187903,
    8742532579937987008,
    78143684950290654,
    10454925311792498315,
    7789818152192856725,
    3486011543032592030,
    17188770042768805161,
    10490412495468775616,
    298640180115056798,
    12895819509602002088,
    1755013598313843104,
    17242416429764373372,
    993835663551930043,
    17604339535769584753,
    17954116481891390155,
    332811330083846624,
    14730023810555747819,
    435413210797820565,
    1781261080337413422,
    4148505421656051973,
    980199695323775177,
    4706730905557535223,
    12734714246714791746,
    14273996233795959868,
    7921735635146743134,
    14772166129594741813,
    2171393332099124215,
    11431591906353698662,
    1968460689143086961,
    12435956952300281356,
    18203712123938736914,
    13226878153002754824,
    4722189513468037980,
    14552059159516237140,
    2186026037853355566,
    11286141841507813990,
    565856028734827369,
    13655906686104936396,
    8559867348362880285,
    2797343365604350633,
    4465794635391355875,
    10602340776590577912,
    6532765362293732644,
    9971594382705594993,
    8246981798349136173,
    4260734168634971109,
    3096607081570771,
    823237991393038853,
    17532689952600815755,
    12134755733102166916,
    10570439735096051664,
    18403803913856082900,
    13128404168847275462,
    16663835358650929116,
    16546671721888068220,
    4685011688485137218,
    1959001578540316019,
    16340711608595843821,
    9460495021221259854,
    3858517940845573321,
    9427670160758976948,
    18064975260450261693,
    4905506444249847758,
    15986418616213903133,
    9282818778268010424,
    9769107232941785010,
    8521948467436343364,
    7419602577337727529,
    5926710664024036226,
    11667040483862285999,
    12291037072726747355,
    12257844845576909578,
    5216888292865522221,
    4949589496388892504,
    6571373688631618567,
    10091372984903831417,
    6240610640427541397,
    6328690792776976228,
    11836184983048970818,
    12710419323566440454,
    10374451385652807364,
    8254232795575550118,
    9866490979395302091,
    12991014125893242232,
    1063347186953727863,
    2952135743830082310,
    17315974856538709017,
    14554512349953922358,
    14134347382797855179,
    17882046380988406016,
    17463193400175360824,
    3726957756828900632,
    17604631050958608669,
    7585987025945897953,
    14470977033142357695,
    10643295498661723800,
    8871197056529643534,
    8384208064507509379,
    9280566467635869786,
    87319369282683875,
    1100172740622998121,
    622721254307916221,
    16843330035110191506,
    13024130485811341782,
    12334996107415540952,
    461552745543935046,
    8140793910765831499,
    9008477689109468885,
    17409910369122253035,
    1804565454784197696,
    5310948951638903141,
    12531953612536647976,
    6147853502869470889,
    1125351356112285953,
    6467901683012265601,
    16792548587138841945,
    14092833521360698433,
    13651748079341829335,
    10688258556205752814,
    1823953496327460008,
    2558053704584850519,
    13269131806718310421,
    4608410977522599149,
    9221187654763620553,
    4611978991500182874,
    8855429001286425455,
    5696709580182222832,
    17579496245625003067,
    5267934104348282564,
    1835676094870249003,
    3542280417783105151,
    11824126253481498070,
    9504622962336320170,
    17887320494921151801,
    6574518722274623914,
    16658124633332643846,
    13808019273382263890,
    13092903038683672100,
    501471167473345282,
    11161560208140424921,
    13001827442679699140,
    14739684132127818993,
    2868223407847949089,
    1726410909424820290,
    6794531346610991076,
    6698331109000773276,
    3680934785728193940,
    8875468921351982841,
    5477651765997654015,
    12280771278642823764,
    3619998794343148112,
    6883119128428826230,
    13512760119042878827,
    3675597821767844913,
    5414638790278102151,
    3587251244316549755,
    17100313981528550060,
    11048426899172804713,
    1396562484529002856,
    2252873797267794672,
    14201526079271439737,
    16618356769072634008,
    144564843743666734,
    11912794688498369701,
    10937102025343594422,
    15432144252435329607,
    2221546737981282133,
    6015808993571140081,
    7447996510907844453,
    7039231904611782781,
    2218118803134364409,
    9472427559993341443,
    11066826455107746221,
    6223571389973384864,
    13615228926415811268,
    10241352486499609335,
    12605380114102527595,
    11403123666082872720,
    9771232158486004346,
    11862860570670038891,
    10489319728736503343,
    588166220336712628,
    524399652036013851,
    2215268375273320892,
    1424724725807107497,
    2223952838426612865,
    1901666565705039600,
    14666084855112001547,
    16529527081633002035,
    3475787534446449190,
    17395838083455569055,
    10036301139275236437,
    5830062976180250577,
    6201110308815839738,
    3908827014617539568,
    13269427316630307104,
    1104974093011983663,
    335137437077264843,
    13411663683768112565,
    7907493007733959147,
    17240291213488173803,
    6357405277112016289,
    7875258449007392338,
    16100900298327085499,
    13542432207857463387,
    9466802464896264825,
    9221606791343926561,
    10417300838622453849,
    13201838829839066427,
    9833345239958202067,
    16688814355354359676,
    13315432437333533951,
    378443609734580293,
    14654525144709164243,
    1967217494445269914,
    16045947041840686058,
    18049263629128746044,
    1957063364541610677,
    16123386013589472221,
    5923137592664329389,
    12399617421793397670,
    3403518680407886401,
    6416516714555000604,
    13286977196258324106,
    17641011370212535641,
    14823578540420219384,
    11909888788340877523,
    11040604022089158722,
    14682783085930648838,
    7896655986299558210,
    9328642557612914244,
    6213125364180629684,
    16259136970573308007,
    12025260496935037210,
    1512031407150257270,
    1295709332547428576,
    13851880110872460625,
    6734559515296147531,
    17720805166223714561,
    11264121550751120724,
    7210341680607060660,
    17759718475616004694,
    610155440804635364,
    3209025413915748371,
];

pub const ROUND_CONSTANTS: [u64; NUM_ROUNDS * STATE_SIZE] = [
    3006656781416918236,
    4369161505641058227,
    6684374425476535479,
    15779820574306927140,
    9604497860052635077,
    6451419160553310210,
    16926195364602274076,
    6738541355147603274,
    13653823767463659393,
    16331310420018519380,
    10921208506902903237,
    5856388654420905056,
    180518533287168595,
    6394055120127805757,
    4624620449883041133,
    4245779370310492662,
    11436753067664141475,
    9565904130524743243,
    1795462928700216574,
    6069083569854718822,
    16847768509740167846,
    4958030292488314453,
    6638656158077421079,
    7387994719600814898,
    1380138540257684527,
    2756275326704598308,
    6162254851582803897,
    4357202747710082448,
    12150731779910470904,
    3121517886069239079,
    14951334357190345445,
    11174705360936334066,
    17619090104023680035,
    9879300494565649603,
    6833140673689496042,
    8026685634318089317,
    6481786893261067369,
    15148392398843394510,
    11231860157121869734,
    2645253741394956018,
    15345701758979398253,
    1715545688795694261,
    3419893440622363282,
    12314745080283886274,
    16173382637268011204,
    2012426895438224656,
    6886681868854518019,
    9323151312904004776,
    14061124303940833928,
    14720644192628944300,
    3643016909963520634,
    15164487940674916922,
    18095609311840631082,
    17450128049477479068,
    13770238146408051799,
    959547712344137104,
    12896174981045071755,
    15673600445734665670,
    5421724936277706559,
    15147580014608980436,
    10475549030802107253,
    9781768648599053415,
    12208559126136453589,
    14883846462224929329,
    4104889747365723917,
    748723978556009523,
    1227256388689532469,
    5479813539795083611,
    8771502115864637772,
    16732275956403307541,
    4416407293527364014,
    828170020209737786,
    12657110237330569793,
    6054985640939410036,
    4339925773473390539,
    12523290846763939879,
    6515670251745069817,
    3304839395869669984,
    13139364704983394567,
    7310284340158351735,
    10864373318031796808,
    17752126773383161797,
    1934077736434853411,
    12181011551355087129,
    16512655861290250275,
    17788869165454339633,
    12226346139665475316,
    521307319751404755,
    18194723210928015140,
    11017703779172233841,
    15109417014344088693,
    16118100307150379696,
    16104548432406078622,
    10637262801060241057,
    10146828954247700859,
    14927431817078997000,
    8849391379213793752,
    14873391436448856814,
    15301636286727658488,
    14600930856978269524,
    14900320206081752612,
    9439125422122803926,
    17731778886181971775,
    11364016993846997841,
    11610707911054206249,
    16438527050768899002,
    1230592087960588528,
    11390503834342845303,
    10608561066917009324,
    5454068995870010477,
    13783920070953012756,
    10807833173700567220,
];

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Gf65536(u16);

impl Add for Gf65536 {
    type Output = Gf65536;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

#[inline]
fn slow_mul(lhs: u32, rhs: u32) -> u32 {
    let mut product = 0;
    for i in 0..16 {
        for j in 0..16 {
            product ^= (lhs << j) & (rhs << i) & (1 << (i + j));
        }
    }
    product
}

impl Mul for Gf65536 {
    type Output = Gf65536;

    fn mul(self, rhs: Self) -> Self::Output {
        let reduction_table: [u32; 16] = [
            45, 90, 180, 360, 720, 1440, 2880, 5760, 11520, 23040, 46080, 26669, 53338, 41113,
            16671, 33342,
        ];
        let mut product = slow_mul(self.0 as u32, rhs.0 as u32);
        // let mut product = karatsuba(16, self.0 as u32, rhs.0 as u32);
        for (i, red) in reduction_table.into_iter().enumerate() {
            if product & (1 << (16 + i)) != 0 {
                product ^= red;
            }
        }
        Gf65536((product & 65535u32).try_into().unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tip5State {
    pub state: [BFieldElement; STATE_SIZE],
}

impl Tip5State {
    fn new() -> Tip5State {
        Tip5State {
            state: [BFieldElement::zero(); STATE_SIZE],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tip5 {
    lookup_table: [u16; 65536],
    mds: [BFieldElement; STATE_SIZE],
    mds_ntt: [BFieldElement; STATE_SIZE],
    mds_swapped: [BFieldElement; STATE_SIZE],
    log2_state_size: usize,
    omega: BFieldElement,
    omega_inverse: BFieldElement,
    powers_of_omega: Vec<BFieldElement>,
    powers_of_omega_inverse: Vec<BFieldElement>,
    powers_of_omega_bitreversed: Vec<BFieldElement>,
    powers_of_omega_inverse_bitreversed: Vec<BFieldElement>,
}

impl Tip5 {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut lookup_table = [0u16; 65536];
        let log2_state_size = 4usize;
        for i in 0..=u16::MAX {
            let gfe = Gf65536(i);
            let cubed = gfe * gfe * gfe;
            lookup_table[i as usize] = cubed.0;
        }
        let omega = BFieldElement::primitive_root_of_unity(STATE_SIZE as u64).unwrap();
        let omega_inverse = omega.inverse();

        let mds: [BFieldElement; STATE_SIZE] = [
            256, 8192, 2, 1024, 1, 268436456, 1, 4194304, 524288, 16, 8, 128, 16777216, 2048,
            1073741824, 2,
        ]
        .map(BFieldElement::new);

        // pre-compute powers of omega
        // let w_m = omega.mod_pow_u32(n / (2 * m)); where n = length and m = 1, 2, 4, ... < n
        let powers_of_omega: Vec<BFieldElement> = (0..log2_state_size)
            .map(|l| 1 << l)
            .map(|m| STATE_SIZE / (2 * m))
            .map(|e| omega.mod_pow(e as u64))
            .collect();
        let powers_of_omega_inverse: Vec<BFieldElement> = (0..log2_state_size)
            .map(|l| 1 << l)
            .map(|m| STATE_SIZE / (2 * m))
            .map(|e| omega_inverse.mod_pow(e as u64))
            .collect();
        let all_powers_of_omega: Vec<BFieldElement> = (0..STATE_SIZE)
            .map(|e| omega.mod_pow_u32(e as u32))
            .collect();
        let powers_of_omega_bitreversed: Vec<BFieldElement> = (0..STATE_SIZE)
            .map(|n| Self::bitreverse(n as usize, log2_state_size))
            .map(|reversed_index| all_powers_of_omega[reversed_index as usize])
            .collect();
        let powers_of_omega_inverse_bitreversed: Vec<BFieldElement> = (0..STATE_SIZE)
            .map(|n| Self::bitreverse(n as usize, log2_state_size))
            .map(|reversed_index| all_powers_of_omega[reversed_index as usize].inverse())
            .collect();

        let mut mds_ntt: [BFieldElement; STATE_SIZE] = mds.to_vec().try_into().unwrap();
        Self::ntt_withswap(&mut mds_ntt, omega, log2_state_size);

        let mut mds_swapped: [BFieldElement; STATE_SIZE] = mds.to_vec().try_into().unwrap();
        Self::ntt_noswap(&mut mds_swapped);

        assert_eq!(1 << log2_state_size, STATE_SIZE);

        Self {
            lookup_table,
            mds,
            mds_ntt,
            mds_swapped,
            log2_state_size,
            omega,
            omega_inverse,
            powers_of_omega,
            powers_of_omega_inverse,
            powers_of_omega_bitreversed,
            powers_of_omega_inverse_bitreversed,
        }
    }

    #[inline]
    fn fermat_cube_map(x: u32) -> u32 {
        let x2 = x * x;
        let x2hi = x2 >> 16;
        let x2lo = x2 & 0xffff;
        let x2p = x2lo + u32::from(x2lo < x2hi) * 65537 - x2hi;
        let x3 = x2p * x;
        let x3hi = x3 >> 16;
        let x3lo = x3 & 0xffff;
        x3lo + u32::from(x3lo < x3hi) * 65537 - x3hi
    }

    #[inline]
    fn inverted_fermat_cube_map(x: u32) -> u32 {
        65536 - Self::fermat_cube_map(65535 - x)
    }

    #[inline]
    fn sbox(&self, element: &mut BFieldElement) -> BFieldElement {
        let value = element.value();

        let a: u32 = (value >> 48).try_into().unwrap();
        let b: u32 = ((value >> 32) & 0xffff).try_into().unwrap();
        let c: u32 = ((value >> 16) & 0xffff).try_into().unwrap();
        let d: u32 = (value & 0xffff).try_into().unwrap();

        // let a_ = 65535 - self.lookup_table[(65535 - a) as usize];
        // let b_ = 65535 - self.lookup_table[(65535 - b) as usize];
        // let c_ = self.lookup_table[c as usize];
        // let d_ = self.lookup_table[d as usize];

        let a_ = Self::inverted_fermat_cube_map(a);
        let b_ = Self::inverted_fermat_cube_map(b);
        let c_ = Self::fermat_cube_map(c);
        let d_ = Self::fermat_cube_map(d);

        BFieldElement::new(
            ((a_ as u64) << 48) | ((b_ as u64) << 32) | ((c_ as u64) << 16) | (d_ as u64),
        )
    }

    #[inline]
    fn bitreverse(mut t: usize, log2_of_n: usize) -> usize {
        let mut r = 0;
        for _ in 0..log2_of_n {
            r = (r << 1) | (t & 1);
            t >>= 1;
        }
        r
    }

    #[allow(clippy::many_single_char_names)]
    fn ntt_withswap(x: &mut [BFieldElement], omega: BFieldElement, log_2_of_n: usize) {
        Self::bitreverse_order(x);

        let mut m: usize = 1;
        for i in 0..log_2_of_n as usize {
            let w_m = omega.mod_pow_u32((STATE_SIZE / (2 * m)).try_into().unwrap());
            // let w_m = powers_of_omega[i as usize];
            println!("omega {}: {}", i, w_m);
            let mut k: usize = 0;
            while k < STATE_SIZE as usize {
                let mut w = BFieldElement::one();
                for j in 0..m {
                    let mut t = x[(k + j + m) as usize];
                    t *= w;
                    let mut tmp = x[(k + j) as usize];
                    tmp -= t;
                    x[(k + j + m) as usize] = tmp;
                    x[(k + j) as usize] += t;
                    w *= w_m;
                }

                k += 2 * m;
            }

            m *= 2;
        }
    }

    fn bitreverse_order(array: &mut [BFieldElement]) {
        for k in 0..STATE_SIZE {
            let rk = Self::bitreverse(k, 4);
            if k < rk {
                array.swap(rk, k);
            }
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn ntt_noswap(x: &mut [BFieldElement]) {
        const POWERS_OF_OMEGA_BITREVERSED: [BFieldElement; 8] = [
            BFieldElement::new(1),
            BFieldElement::new(281474976710656),
            BFieldElement::new(18446744069397807105),
            BFieldElement::new(18446742969902956801),
            BFieldElement::new(17293822564807737345),
            BFieldElement::new(4096),
            BFieldElement::new(4503599626321920),
            BFieldElement::new(18446744000695107585),
        ];

        // outer loop iteration 1
        for j in 0..8 {
            let u = x[j];
            let v = x[j + 8] * BFieldElement::one();
            x[j] = u + v;
            x[j + 8] = u - v;
        }

        // outer loop iteration 2
        for (i, zeta) in POWERS_OF_OMEGA_BITREVERSED.iter().enumerate().take(2) {
            let s = i * 8;
            for j in s..(s + 4) {
                let u = x[j];
                let v = x[j + 4] * *zeta;
                x[j] = u + v;
                x[j + 4] = u - v;
            }
        }

        // outer loop iteration 3
        for (i, zeta) in POWERS_OF_OMEGA_BITREVERSED.iter().enumerate().take(4) {
            let s = i * 4;
            for j in s..(s + 2) {
                let u = x[j];
                let v = x[j + 2] * *zeta;
                x[j] = u + v;
                x[j + 2] = u - v;
            }
        }

        // outer loop iteration 4
        for (i, zeta) in POWERS_OF_OMEGA_BITREVERSED.iter().enumerate().take(8) {
            let s = i * 2;
            let u = x[s];
            let v = x[s + 1] * *zeta;
            x[s] = u + v;
            x[s + 1] = u - v;
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn intt_noswap(x: &mut [BFieldElement]) {
        const POWERS_OF_OMEGA_INVERSE: [BFieldElement; 8] = [
            BFieldElement::new(1),
            BFieldElement::new(68719476736),
            BFieldElement::new(1099511627520),
            BFieldElement::new(18446744069414580225),
            BFieldElement::new(18446462594437873665),
            BFieldElement::new(18442240469788262401),
            BFieldElement::new(16777216),
            BFieldElement::new(1152921504606846976),
        ];

        // outer loop iteration 1
        {
            // while k < STATE_SIZE as usize
            // inner loop iteration 1
            {
                let u = x[1];
                let v = x[0];
                x[1] = v - u;
                x[0] = v + u;
            }

            // inner loop iteration 2
            {
                let u = x[2 + 1];
                let v = x[2];
                x[2 + 1] = v - u;
                x[2] = v + u;
            }

            // inner loop iteration 3
            {
                let u = x[4 + 1];
                let v = x[4];
                x[4 + 1] = v - u;
                x[4] = v + u;
            }

            // inner loop iteration 4
            {
                let u = x[6 + 1];
                let v = x[6];
                x[6 + 1] = v - u;
                x[6] = v + u;
            }

            // inner loop iteration 5
            {
                let u = x[8 + 1];
                let v = x[8];
                x[8 + 1] = v - u;
                x[8] = v + u;
            }

            // inner loop iteration 6
            {
                let u = x[10 + 1];
                let v = x[10];
                x[10 + 1] = v - u;
                x[10] = v + u;
            }

            // inner loop iteration 7
            {
                let u = x[12 + 1];
                let v = x[12];
                x[12 + 1] = v - u;
                x[12] = v + u;
            }

            // inner loop iteration 7
            {
                let u = x[14 + 1];
                let v = x[14];
                x[14 + 1] = v - u;
                x[14] = v + u;
            }
        }

        // outer loop iteration 2
        {
            // while k < STATE_SIZE as usize
            // inner loop iteration 1
            {
                for j in 0..2 {
                    let zeta = POWERS_OF_OMEGA_INVERSE[4 * j];
                    {
                        let u = x[j + 2] * zeta;
                        let v = x[j];
                        x[j + 2] = v - u;
                        x[j] = v + u;
                    }
                    // inner loop iteration 2
                    {
                        let u = x[4 + j + 2] * zeta;
                        let v = x[4 + j];
                        x[4 + j + 2] = v - u;
                        x[4 + j] = v + u;
                    }
                    // inner loop iteration 3
                    {
                        let u = x[8 + j + 2] * zeta;
                        let v = x[8 + j];
                        x[8 + j + 2] = v - u;
                        x[8 + j] = v + u;
                    }
                    // inner loop iteration 4
                    {
                        let u = x[12 + j + 2] * zeta;
                        let v = x[12 + j];
                        x[12 + j + 2] = v - u;
                        x[12 + j] = v + u;
                    }
                }
            }
        }

        // outer loop iteration 3
        {
            // while k < STATE_SIZE as usize
            {
                for j in 0..4 {
                    let zeta = POWERS_OF_OMEGA_INVERSE[2 * j];
                    // inner loop iteration 1
                    {
                        let u = x[j + 4] * zeta;
                        let v = x[j];
                        x[j + 4] = v - u;
                        x[j] = v + u;
                    }
                    // inner loop iteration 2
                    {
                        let u = x[8 + j + 4] * zeta;
                        let v = x[8 + j];
                        x[8 + j + 4] = v - u;
                        x[8 + j] = v + u;
                    }
                }
            }
        }

        // outer loop iteration 4
        {
            for j in 0..8 {
                let zeta = POWERS_OF_OMEGA_INVERSE[j];
                let u = x[j + 8] * zeta;
                let v = x[j];
                x[j + 8] = v - u;
                x[j] = v + u;
            }
        }
    }

    pub fn mul_state(state: &mut [BFieldElement; STATE_SIZE], arg: BFieldElement) {
        state.iter_mut().for_each(|s| *s *= arg);
    }

    #[inline]
    pub fn mds_ntt(&self, state: &mut [BFieldElement; STATE_SIZE]) {
        ntt(state, self.omega, self.log2_state_size as u32);
        for (i, m) in self.mds_ntt.iter().enumerate() {
            state[i] *= *m;
        }
        ntt(state, self.omega_inverse, self.log2_state_size as u32);
    }

    #[inline]
    pub fn mds_withswap(&self, state: &mut [BFieldElement; STATE_SIZE]) {
        Self::ntt_withswap(state, self.omega, self.log2_state_size);
        for (i, m) in self.mds_ntt.iter().enumerate() {
            state[i] *= *m;
        }
        Self::ntt_withswap(state, self.omega_inverse, self.log2_state_size);
    }

    #[inline]
    pub fn mds_noswap(&self, state: &mut [BFieldElement; STATE_SIZE]) {
        Self::ntt_noswap(state);

        for (i, m) in self.mds_swapped.iter().enumerate() {
            state[i] *= *m;
        }

        Self::intt_noswap(state);
    }

    pub fn mds_schoolbook(&self, state: &mut [BFieldElement; STATE_SIZE]) {
        let mut array = [BFieldElement::zero(); 2 * STATE_SIZE];
        for i in 0..STATE_SIZE {
            for j in 0..STATE_SIZE {
                array[i + j] += state[i] * self.mds[j];
            }
        }
        for i in 0..STATE_SIZE {
            state[i] = array[i] + array[STATE_SIZE + i];
        }
        Self::mul_state(state, BFieldElement::new(STATE_SIZE as u64));
    }

    pub fn mds_polynomial(&self, state: &mut [BFieldElement; STATE_SIZE]) {
        let a = Polynomial::new(state.to_vec());
        let b = Polynomial::new(self.mds.to_vec());
        let m = Polynomial::new(vec![BFieldElement::zero(), BFieldElement::one()])
            .mod_pow(BigInt::from(STATE_SIZE))
            - Polynomial::<BFieldElement>::one();
        let coeffs = ((a * b) % m).coefficients;
        state[..STATE_SIZE].copy_from_slice(&coeffs[..STATE_SIZE]);
        Self::mul_state(state, BFieldElement::new(STATE_SIZE as u64));
    }

    #[inline]
    fn round(&self, sponge: &mut Tip5State, round_index: usize) {
        // S-box
        for i in 0..STATE_SIZE {
            self.sbox(&mut sponge.state[i]);
        }

        // MDS matrix
        // let mut v: [BFieldElement; STATE_SIZE] = [BFieldElement::from(0u64); STATE_SIZE];
        // for i in 0..STATE_SIZE {
        //     for j in 0..STATE_SIZE {
        //         v[i] += BFieldElement::from(MDS[i * STATE_SIZE + j]) * sponge.state[j];
        //     }
        // }
        // sponge.state = v;
        self.mds_noswap(&mut sponge.state);

        // round constants A
        for i in 0..STATE_SIZE {
            sponge.state[i] += BFieldElement::from(ROUND_CONSTANTS[round_index * STATE_SIZE + i]);
        }
    }

    // permutation
    fn permutation(&self, sponge: &mut Tip5State) {
        for i in 0..NUM_ROUNDS {
            self.round(sponge, i);
        }
    }

    /// hash_10
    /// Hash 10 elements, or two digests. There is no padding because
    /// the input length is fixed.
    pub fn hash_10(&self, input: &[BFieldElement; 10]) -> [BFieldElement; 5] {
        let mut sponge = Tip5State::new();

        // absorb once
        sponge.state[..10].copy_from_slice(input);

        // apply domain separation for fixed-length input
        sponge.state[10] = BFieldElement::one();

        // apply permutation
        self.permutation(&mut sponge);

        // squeeze once
        sponge.state[..5].try_into().unwrap()
    }

    /// hash_varlen hashes an arbitrary number of field elements.
    ///
    /// Takes care of padding by applying the padding rule: append a single 1 ∈ Fp
    /// and as many 0 ∈ Fp elements as required to make the number of input elements
    /// a multiple of `RATE`.
    pub fn hash_varlen(&self, input: &[BFieldElement]) -> [BFieldElement; 5] {
        let mut sponge = Tip5State::new();

        // pad input
        let mut padded_input = input.to_vec();
        padded_input.push(BFieldElement::one());
        while padded_input.len() % RATE != 0 {
            padded_input.push(BFieldElement::zero());
        }

        // absorb
        while !padded_input.is_empty() {
            for (sponge_state_element, input_element) in sponge
                .state
                .iter_mut()
                .take(RATE)
                .zip_eq(padded_input.iter().take(RATE))
            {
                *sponge_state_element += input_element.to_owned();
            }
            padded_input.drain(..RATE);
            self.permutation(&mut sponge);
        }

        // squeeze once
        sponge.state[..5].try_into().unwrap()
    }
}

#[cfg(test)]
mod tip5_tests {
    use rayon::prelude::{IntoParallelIterator, ParallelIterator};

    use crate::shared_math::{
        b_field_element::BFieldElement,
        other::random_elements,
        rescue_prime_regular::STATE_SIZE,
        tip5::{Gf65536, Tip5},
    };

    #[test]
    fn test_gf65536_multiply() {
        assert_eq!(Gf65536(4840) * Gf65536(63660), Gf65536(46656));
        assert_eq!(Gf65536(9489) * Gf65536(24015), Gf65536(46252));
        assert_eq!(Gf65536(40948) * Gf65536(42030), Gf65536(2363));
        assert_eq!(Gf65536(12138) * Gf65536(27906), Gf65536(47471));
        assert_eq!(Gf65536(22030) * Gf65536(3193), Gf65536(57710));
        assert_eq!(Gf65536(4028) * Gf65536(59802), Gf65536(5087));
        assert_eq!(Gf65536(38495) * Gf65536(57103), Gf65536(29308));
        assert_eq!(Gf65536(27309) * Gf65536(41572), Gf65536(11130));
        assert_eq!(Gf65536(32647) * Gf65536(9294), Gf65536(25696));
        assert_eq!(Gf65536(64633) * Gf65536(51650), Gf65536(2540));
        assert_eq!(Gf65536(39639) * Gf65536(12237), Gf65536(58640));
        assert_eq!(Gf65536(62298) * Gf65536(33374), Gf65536(45596));
        assert_eq!(Gf65536(20672) * Gf65536(53133), Gf65536(25797));
        assert_eq!(Gf65536(46126) * Gf65536(38148), Gf65536(18426));
        assert_eq!(Gf65536(34416) * Gf65536(23362), Gf65536(62190));
        assert_eq!(Gf65536(53588) * Gf65536(28348), Gf65536(7948));
        assert_eq!(Gf65536(30702) * Gf65536(61497), Gf65536(62995));
        assert_eq!(Gf65536(55882) * Gf65536(56671), Gf65536(7538));
        assert_eq!(Gf65536(1083) * Gf65536(57433), Gf65536(11098));
        assert_eq!(Gf65536(1101) * Gf65536(49676), Gf65536(6365));
    }

    #[inline]
    fn fermat_cube_map(x: u32) -> u32 {
        let x2 = x * x;
        let x2hi = x2 >> 16;
        let x2lo = x2 & 0xffff;
        let x2p = x2lo + u32::from(x2lo < x2hi) * 65537 - x2hi;
        let x3 = x2p * x;
        let x3hi = x3 >> 16;
        let x3lo = x3 & 0xffff;
        x3lo + u32::from(x3lo < x3hi) * 65537 - x3hi
    }

    #[inline]
    fn inverted_fermat_cube_map(x: u32) -> u32 {
        65535 - fermat_cube_map(65535 - x)
    }

    #[test]
    #[ignore = "used for calculating parameters"]
    fn test_fermat_cube_map_is_permutation() {
        let mut touched = [false; 65536];
        for i in 0..65536 {
            touched[fermat_cube_map(i) as usize] = true;
        }
        assert!(touched.iter().all(|t| *t));
        assert_eq!(fermat_cube_map(0), 0);
    }

    #[test]
    #[ignore = "used for calculating parameters"]
    fn test_inverted_fermat_cube_map_is_permutation() {
        let mut touched = [false; 65536];
        for i in 0..65536 {
            touched[inverted_fermat_cube_map(i) as usize] = true;
        }
        assert!(touched.iter().all(|t| *t));
        assert_eq!(inverted_fermat_cube_map(65535), 65535);
    }

    #[test]
    #[ignore = "used for calculating parameters"]
    fn calculate_differential_uniformity() {
        // cargo test calculate_differential_uniformity -- --include-ignored --nocapture
        let count_satisfiers_fermat = |a, b| {
            (0..(1 << 16))
                .map(|x| {
                    u32::from(
                        (0xffff + fermat_cube_map((x + a) & 0xffff) - fermat_cube_map(x)) & 0xffff
                            == b,
                    )
                })
                .sum()
        };
        let du_fermat: u32 = (1..65536)
            .into_par_iter()
            .map(|a| {
                (1..65536)
                    .into_iter()
                    .map(|b| count_satisfiers_fermat(a, b))
                    .max()
                    .unwrap()
            })
            .max()
            .unwrap();
        println!("differential uniformity for fermat cube map: {}", du_fermat);

        let count_satisfiers_inverted = |a, b| {
            (0..(1 << 16))
                .map(|x| {
                    u32::from(
                        (0xffff + inverted_fermat_cube_map((x + a) & 0xffff)
                            - inverted_fermat_cube_map(x))
                            & 0xffff
                            == b,
                    )
                })
                .sum()
        };
        let du_inverted: u32 = (1..65536)
            .into_par_iter()
            .map(|a| {
                (1..65536)
                    .into_iter()
                    .map(|b| count_satisfiers_inverted(a, b))
                    .max()
                    .unwrap()
            })
            .max()
            .unwrap();
        println!(
            "differential uniformity for fermat cube map: {}",
            du_inverted
        );
    }

    #[test]
    fn mds_match() {
        let mut ntt_: [BFieldElement; STATE_SIZE] = random_elements(16).try_into().unwrap();
        let mut withswap_: [BFieldElement; STATE_SIZE] = ntt_;
        let mut no_swap: [BFieldElement; STATE_SIZE] = withswap_;
        let mut schoolbook_: [BFieldElement; STATE_SIZE] = no_swap;
        let mut polynomial_: [BFieldElement; STATE_SIZE] = schoolbook_;

        let tip5 = Tip5::new();

        tip5.mds_ntt(&mut ntt_);
        tip5.mds_withswap(&mut withswap_);
        tip5.mds_noswap(&mut no_swap);
        tip5.mds_polynomial(&mut schoolbook_);
        tip5.mds_schoolbook(&mut polynomial_);
        let mut fails = false;
        if ntt_ != withswap_ {
            println!("ntt =/= withswap");
            fails = true;
        }
        if withswap_ != no_swap {
            println!("withswap =/= noswap");
            fails = true;
        }
        if no_swap != schoolbook_ {
            println!("noswap =/= schoolbook");
            fails = true;
        }
        if schoolbook_ != polynomial_ {
            println!("schoolbook =/= polynomial");
            fails = true;
        }
        if polynomial_ != ntt_ {
            println!("polynomial =/= ntt");
            fails = true;
        }
        // assert_eq!(ntt_, withswap_, "ntt =/= withswap");
        // assert_eq!(schoolbook_, polynomial_, "schoolbook =/= polynomial");
        // assert_eq!(withswap_, schoolbook_, "withswap =/= schoolbook");
        // assert_eq!(withswap_, no_swap, "withswap =/= noswap");
        // assert_eq!(no_swap, schoolbook_, "noswap =/= schoolbook");
        assert!(!fails);

        for m in tip5.mds_swapped {
            println!("{}", m.value());
        }
    }
}
