import Foundation
import Testing

@testable import AlgoModels

@Test func decodePayment() throws {
    let uint8Array: [UInt8] = [
        84,
        88,
        137,
        163,
        97,
        109,
        116,
        205,
        3,
        232,
        163,
        102,
        101,
        101,
        205,
        3,
        232,
        162,
        102,
        118,
        206,
        2,
        206,
        143,
        251,
        163,
        103,
        101,
        110,
        172,
        116,
        101,
        115,
        116,
        110,
        101,
        116,
        45,
        118,
        49,
        46,
        48,
        162,
        103,
        104,
        196,
        32,
        72,
        99,
        181,
        24,
        164,
        179,
        200,
        78,
        200,
        16,
        242,
        45,
        79,
        16,
        129,
        203,
        15,
        113,
        240,
        89,
        167,
        172,
        32,
        222,
        198,
        47,
        127,
        112,
        229,
        9,
        58,
        34,
        162,
        108,
        118,
        206,
        2,
        206,
        147,
        227,
        163,
        114,
        99,
        118,
        196,
        32,
        250,
        192,
        119,
        12,
        168,
        98,
        69,
        105,
        178,
        165,
        55,
        223,
        154,
        187,
        123,
        50,
        13,
        38,
        59,
        171,
        71,
        198,
        255,
        73,
        50,
        202,
        202,
        57,
        38,
        47,
        242,
        134,
        163,
        115,
        110,
        100,
        196,
        32,
        193,
        17,
        25,
        40,
        226,
        19,
        41,
        179,
        120,
        65,
        13,
        106,
        47,
        7,
        115,
        224,
        235,
        25,
        223,
        21,
        129,
        160,
        185,
        4,
        203,
        142,
        61,
        187,
        254,
        41,
        187,
        56,
        164,
        116,
        121,
        112,
        101,
        163,
        112,
        97,
        121,
    ]
    let payment = try AlgoModels.decodeTransaction(bytes: Data(uint8Array))
    print(payment)
}
