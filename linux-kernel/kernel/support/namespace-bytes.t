use language (version is v0.1, features is (systems))

pub word is fn ((region : KernelByteRegion, offset : Nat)) -> Nat
  region unsigned load (offset-bytes is offset, byte-count is 8, byte-order is little-endian)
    Some value then value
    None then 18446744073709551616

input-byte is fn ((region : KernelByteRegion, index : Nat)) -> Nat
  offset is 3200 + index
  region unsigned load (offset-bytes is offset, byte-count is 1, byte-order is little-endian)
    Some value then value
    None then 256

# Fixed field positions are source layout constants, not runtime address math.
both is fn ((first : Boolean, second : Boolean)) -> Boolean
  first
    true then second
    false then false

character is fn (value : Nat) -> Boolean
  positive is value > 0
  bounded is value <= 255
  normal is value != 47
  both (positive, both (bounded, normal))

valid-value is fn ((value : Nat, index : Nat, length : Nat)) -> Boolean
  index < length
    true then character value
    false then value = 0

valid-byte is fn ((region : KernelByteRegion, offset : Nat, index : Nat, length : Nat)) -> Boolean
  region unsigned load (offset-bytes is offset, byte-count is 1, byte-order is little-endian)
    Some value then valid-value (value, index, length)
    None then false

all-four is fn ((first : Boolean, second : Boolean, third : Boolean, fourth : Boolean)) -> Boolean
  left is both (first, second)
  right is both (third, fourth)
  both (left, right)

valid-payload is fn ((region : KernelByteRegion, length : Nat)) -> Boolean
  seen-0 is all-four (valid-byte (region, 3200, 0, length), valid-byte (region, 3201, 1, length), valid-byte (region, 3202, 2, length), valid-byte (region, 3203, 3, length))
  seen-1 is all-four (valid-byte (region, 3204, 4, length), valid-byte (region, 3205, 5, length), valid-byte (region, 3206, 6, length), valid-byte (region, 3207, 7, length))
  seen-2 is all-four (valid-byte (region, 3208, 8, length), valid-byte (region, 3209, 9, length), valid-byte (region, 3210, 10, length), valid-byte (region, 3211, 11, length))
  seen-3 is all-four (valid-byte (region, 3212, 12, length), valid-byte (region, 3213, 13, length), valid-byte (region, 3214, 14, length), valid-byte (region, 3215, 15, length))
  seen-4 is all-four (valid-byte (region, 3216, 16, length), valid-byte (region, 3217, 17, length), valid-byte (region, 3218, 18, length), valid-byte (region, 3219, 19, length))
  seen-5 is all-four (valid-byte (region, 3220, 20, length), valid-byte (region, 3221, 21, length), valid-byte (region, 3222, 22, length), valid-byte (region, 3223, 23, length))
  seen-6 is all-four (valid-byte (region, 3224, 24, length), valid-byte (region, 3225, 25, length), valid-byte (region, 3226, 26, length), valid-byte (region, 3227, 27, length))
  seen-7 is all-four (valid-byte (region, 3228, 28, length), valid-byte (region, 3229, 29, length), valid-byte (region, 3230, 30, length), valid-byte (region, 3231, 31, length))
  seen-8 is all-four (valid-byte (region, 3232, 32, length), valid-byte (region, 3233, 33, length), valid-byte (region, 3234, 34, length), valid-byte (region, 3235, 35, length))
  seen-9 is all-four (valid-byte (region, 3236, 36, length), valid-byte (region, 3237, 37, length), valid-byte (region, 3238, 38, length), valid-byte (region, 3239, 39, length))
  seen-10 is all-four (valid-byte (region, 3240, 40, length), valid-byte (region, 3241, 41, length), valid-byte (region, 3242, 42, length), valid-byte (region, 3243, 43, length))
  seen-11 is all-four (valid-byte (region, 3244, 44, length), valid-byte (region, 3245, 45, length), valid-byte (region, 3246, 46, length), valid-byte (region, 3247, 47, length))
  seen-12 is all-four (valid-byte (region, 3248, 48, length), valid-byte (region, 3249, 49, length), valid-byte (region, 3250, 50, length), valid-byte (region, 3251, 51, length))
  seen-13 is all-four (valid-byte (region, 3252, 52, length), valid-byte (region, 3253, 53, length), valid-byte (region, 3254, 54, length), valid-byte (region, 3255, 55, length))
  seen-14 is all-four (valid-byte (region, 3256, 56, length), valid-byte (region, 3257, 57, length), valid-byte (region, 3258, 58, length), valid-byte (region, 3259, 59, length))
  seen-15 is all-four (valid-byte (region, 3260, 60, length), valid-byte (region, 3261, 61, length), valid-byte (region, 3262, 62, length), valid-byte (region, 3263, 63, length))
  seen-16 is all-four (valid-byte (region, 3264, 64, length), valid-byte (region, 3265, 65, length), valid-byte (region, 3266, 66, length), valid-byte (region, 3267, 67, length))
  seen-17 is all-four (valid-byte (region, 3268, 68, length), valid-byte (region, 3269, 69, length), valid-byte (region, 3270, 70, length), valid-byte (region, 3271, 71, length))
  seen-18 is all-four (valid-byte (region, 3272, 72, length), valid-byte (region, 3273, 73, length), valid-byte (region, 3274, 74, length), valid-byte (region, 3275, 75, length))
  seen-19 is all-four (valid-byte (region, 3276, 76, length), valid-byte (region, 3277, 77, length), valid-byte (region, 3278, 78, length), valid-byte (region, 3279, 79, length))
  seen-20 is all-four (valid-byte (region, 3280, 80, length), valid-byte (region, 3281, 81, length), valid-byte (region, 3282, 82, length), valid-byte (region, 3283, 83, length))
  seen-21 is all-four (valid-byte (region, 3284, 84, length), valid-byte (region, 3285, 85, length), valid-byte (region, 3286, 86, length), valid-byte (region, 3287, 87, length))
  seen-22 is all-four (valid-byte (region, 3288, 88, length), valid-byte (region, 3289, 89, length), valid-byte (region, 3290, 90, length), valid-byte (region, 3291, 91, length))
  seen-23 is all-four (valid-byte (region, 3292, 92, length), valid-byte (region, 3293, 93, length), valid-byte (region, 3294, 94, length), valid-byte (region, 3295, 95, length))
  seen-24 is all-four (valid-byte (region, 3296, 96, length), valid-byte (region, 3297, 97, length), valid-byte (region, 3298, 98, length), valid-byte (region, 3299, 99, length))
  seen-25 is all-four (valid-byte (region, 3300, 100, length), valid-byte (region, 3301, 101, length), valid-byte (region, 3302, 102, length), valid-byte (region, 3303, 103, length))
  seen-26 is all-four (valid-byte (region, 3304, 104, length), valid-byte (region, 3305, 105, length), valid-byte (region, 3306, 106, length), valid-byte (region, 3307, 107, length))
  seen-27 is all-four (valid-byte (region, 3308, 108, length), valid-byte (region, 3309, 109, length), valid-byte (region, 3310, 110, length), valid-byte (region, 3311, 111, length))
  seen-28 is all-four (valid-byte (region, 3312, 112, length), valid-byte (region, 3313, 113, length), valid-byte (region, 3314, 114, length), valid-byte (region, 3315, 115, length))
  seen-29 is all-four (valid-byte (region, 3316, 116, length), valid-byte (region, 3317, 117, length), valid-byte (region, 3318, 118, length), valid-byte (region, 3319, 119, length))
  seen-30 is all-four (valid-byte (region, 3320, 120, length), valid-byte (region, 3321, 121, length), valid-byte (region, 3322, 122, length), valid-byte (region, 3323, 123, length))
  seen-31 is all-four (valid-byte (region, 3324, 124, length), valid-byte (region, 3325, 125, length), valid-byte (region, 3326, 126, length), valid-byte (region, 3327, 127, length))
  seen-32 is all-four (valid-byte (region, 3328, 128, length), valid-byte (region, 3329, 129, length), valid-byte (region, 3330, 130, length), valid-byte (region, 3331, 131, length))
  seen-33 is all-four (valid-byte (region, 3332, 132, length), valid-byte (region, 3333, 133, length), valid-byte (region, 3334, 134, length), valid-byte (region, 3335, 135, length))
  seen-34 is all-four (valid-byte (region, 3336, 136, length), valid-byte (region, 3337, 137, length), valid-byte (region, 3338, 138, length), valid-byte (region, 3339, 139, length))
  seen-35 is all-four (valid-byte (region, 3340, 140, length), valid-byte (region, 3341, 141, length), valid-byte (region, 3342, 142, length), valid-byte (region, 3343, 143, length))
  seen-36 is all-four (valid-byte (region, 3344, 144, length), valid-byte (region, 3345, 145, length), valid-byte (region, 3346, 146, length), valid-byte (region, 3347, 147, length))
  seen-37 is all-four (valid-byte (region, 3348, 148, length), valid-byte (region, 3349, 149, length), valid-byte (region, 3350, 150, length), valid-byte (region, 3351, 151, length))
  seen-38 is all-four (valid-byte (region, 3352, 152, length), valid-byte (region, 3353, 153, length), valid-byte (region, 3354, 154, length), valid-byte (region, 3355, 155, length))
  seen-39 is all-four (valid-byte (region, 3356, 156, length), valid-byte (region, 3357, 157, length), valid-byte (region, 3358, 158, length), valid-byte (region, 3359, 159, length))
  seen-40 is all-four (valid-byte (region, 3360, 160, length), valid-byte (region, 3361, 161, length), valid-byte (region, 3362, 162, length), valid-byte (region, 3363, 163, length))
  seen-41 is all-four (valid-byte (region, 3364, 164, length), valid-byte (region, 3365, 165, length), valid-byte (region, 3366, 166, length), valid-byte (region, 3367, 167, length))
  seen-42 is all-four (valid-byte (region, 3368, 168, length), valid-byte (region, 3369, 169, length), valid-byte (region, 3370, 170, length), valid-byte (region, 3371, 171, length))
  seen-43 is all-four (valid-byte (region, 3372, 172, length), valid-byte (region, 3373, 173, length), valid-byte (region, 3374, 174, length), valid-byte (region, 3375, 175, length))
  seen-44 is all-four (valid-byte (region, 3376, 176, length), valid-byte (region, 3377, 177, length), valid-byte (region, 3378, 178, length), valid-byte (region, 3379, 179, length))
  seen-45 is all-four (valid-byte (region, 3380, 180, length), valid-byte (region, 3381, 181, length), valid-byte (region, 3382, 182, length), valid-byte (region, 3383, 183, length))
  seen-46 is all-four (valid-byte (region, 3384, 184, length), valid-byte (region, 3385, 185, length), valid-byte (region, 3386, 186, length), valid-byte (region, 3387, 187, length))
  seen-47 is all-four (valid-byte (region, 3388, 188, length), valid-byte (region, 3389, 189, length), valid-byte (region, 3390, 190, length), valid-byte (region, 3391, 191, length))
  seen-48 is all-four (valid-byte (region, 3392, 192, length), valid-byte (region, 3393, 193, length), valid-byte (region, 3394, 194, length), valid-byte (region, 3395, 195, length))
  seen-49 is all-four (valid-byte (region, 3396, 196, length), valid-byte (region, 3397, 197, length), valid-byte (region, 3398, 198, length), valid-byte (region, 3399, 199, length))
  seen-50 is all-four (valid-byte (region, 3400, 200, length), valid-byte (region, 3401, 201, length), valid-byte (region, 3402, 202, length), valid-byte (region, 3403, 203, length))
  seen-51 is all-four (valid-byte (region, 3404, 204, length), valid-byte (region, 3405, 205, length), valid-byte (region, 3406, 206, length), valid-byte (region, 3407, 207, length))
  seen-52 is all-four (valid-byte (region, 3408, 208, length), valid-byte (region, 3409, 209, length), valid-byte (region, 3410, 210, length), valid-byte (region, 3411, 211, length))
  seen-53 is all-four (valid-byte (region, 3412, 212, length), valid-byte (region, 3413, 213, length), valid-byte (region, 3414, 214, length), valid-byte (region, 3415, 215, length))
  seen-54 is all-four (valid-byte (region, 3416, 216, length), valid-byte (region, 3417, 217, length), valid-byte (region, 3418, 218, length), valid-byte (region, 3419, 219, length))
  seen-55 is all-four (valid-byte (region, 3420, 220, length), valid-byte (region, 3421, 221, length), valid-byte (region, 3422, 222, length), valid-byte (region, 3423, 223, length))
  seen-56 is all-four (valid-byte (region, 3424, 224, length), valid-byte (region, 3425, 225, length), valid-byte (region, 3426, 226, length), valid-byte (region, 3427, 227, length))
  seen-57 is all-four (valid-byte (region, 3428, 228, length), valid-byte (region, 3429, 229, length), valid-byte (region, 3430, 230, length), valid-byte (region, 3431, 231, length))
  seen-58 is all-four (valid-byte (region, 3432, 232, length), valid-byte (region, 3433, 233, length), valid-byte (region, 3434, 234, length), valid-byte (region, 3435, 235, length))
  seen-59 is all-four (valid-byte (region, 3436, 236, length), valid-byte (region, 3437, 237, length), valid-byte (region, 3438, 238, length), valid-byte (region, 3439, 239, length))
  seen-60 is all-four (valid-byte (region, 3440, 240, length), valid-byte (region, 3441, 241, length), valid-byte (region, 3442, 242, length), valid-byte (region, 3443, 243, length))
  seen-61 is all-four (valid-byte (region, 3444, 244, length), valid-byte (region, 3445, 245, length), valid-byte (region, 3446, 246, length), valid-byte (region, 3447, 247, length))
  seen-62 is all-four (valid-byte (region, 3448, 248, length), valid-byte (region, 3449, 249, length), valid-byte (region, 3450, 250, length), valid-byte (region, 3451, 251, length))
  seen-63 is all-four (valid-byte (region, 3452, 252, length), valid-byte (region, 3453, 253, length), valid-byte (region, 3454, 254, length), valid-byte (region, 3455, 255, length))
  checked-0 is all-four (seen-0, seen-1, seen-2, seen-3)
  checked-1 is all-four (seen-4, seen-5, seen-6, seen-7)
  checked-2 is all-four (seen-8, seen-9, seen-10, seen-11)
  checked-3 is all-four (seen-12, seen-13, seen-14, seen-15)
  checked-4 is all-four (seen-16, seen-17, seen-18, seen-19)
  checked-5 is all-four (seen-20, seen-21, seen-22, seen-23)
  checked-6 is all-four (seen-24, seen-25, seen-26, seen-27)
  checked-7 is all-four (seen-28, seen-29, seen-30, seen-31)
  checked-8 is all-four (seen-32, seen-33, seen-34, seen-35)
  checked-9 is all-four (seen-36, seen-37, seen-38, seen-39)
  checked-10 is all-four (seen-40, seen-41, seen-42, seen-43)
  checked-11 is all-four (seen-44, seen-45, seen-46, seen-47)
  checked-12 is all-four (seen-48, seen-49, seen-50, seen-51)
  checked-13 is all-four (seen-52, seen-53, seen-54, seen-55)
  checked-14 is all-four (seen-56, seen-57, seen-58, seen-59)
  checked-15 is all-four (seen-60, seen-61, seen-62, seen-63)
  range-0 is all-four (checked-0, checked-1, checked-2, checked-3)
  range-1 is all-four (checked-4, checked-5, checked-6, checked-7)
  range-2 is all-four (checked-8, checked-9, checked-10, checked-11)
  range-3 is all-four (checked-12, checked-13, checked-14, checked-15)
  all-four (range-0, range-1, range-2, range-3)

not-dot is fn ((region : KernelByteRegion, length : Nat)) -> Boolean
  first is input-byte (region, 0)
  second is input-byte (region, 1)
  one is length = 1
  two is length = 2
  first-dot is first = 46
  second-dot is second = 46
  bad-single is both (one, first-dot)
  bad-pair is both (two, both (first-dot, second-dot))
  bad-single
    true then false
    false then not bad-pair

valid-normal is fn ((region : KernelByteRegion, length : Nat)) -> Boolean
  not-dot (region, length)
    true then valid-payload (region, length)
    false then false

valid-bounded is fn ((region : KernelByteRegion, length : Nat)) -> Boolean
  length <= 255
    true then valid-normal (region, length)
    false then false

pub valid-input is fn ((region : KernelByteRegion, length : Nat)) -> Boolean
  length > 0
    true then valid-bounded (region, length)
    false then false

copy-word is fn ((region : KernelByteRegion, destination : Nat, source : Nat)) -> Boolean
  region unsigned load (offset-bytes is source, byte-count is 8, byte-order is little-endian)
    Some value then region unsigned store (offset-bytes is destination, byte-count is 8, byte-order is little-endian, value is value)
    None then false

clear-word is fn ((region : KernelByteRegion, destination : Nat)) -> Boolean
  region unsigned store (offset-bytes is destination, byte-count is 8, byte-order is little-endian, value is 0)

zero-word is fn ((region : KernelByteRegion, offset : Nat)) -> Boolean
  value is word (region, offset)
  value = 0

copy-pair is fn ((region : KernelByteRegion, destination : Nat, source : Nat)) -> Boolean
  copy-word (region, destination, source)
    true then copy-word (region, destination + 8, source + 8)
    false then false

clear-pair is fn ((region : KernelByteRegion, destination : Nat)) -> Boolean
  clear-word (region, destination)
    true then clear-word (region, destination + 8)
    false then false

zero-pair is fn ((region : KernelByteRegion, offset : Nat)) -> Boolean
  zero-word (region, offset)
    true then zero-word (region, offset + 8)
    false then false

copy-block is fn ((region : KernelByteRegion, destination : Nat, source : Nat)) -> Boolean
  copy-pair (region, destination, source)
    true then copy-pair (region, destination + 16, source + 16)
    false then false

clear-block is fn ((region : KernelByteRegion, destination : Nat)) -> Boolean
  clear-pair (region, destination)
    true then clear-pair (region, destination + 16)
    false then false

zero-block is fn ((region : KernelByteRegion, offset : Nat)) -> Boolean
  zero-pair (region, offset)
    true then zero-pair (region, offset + 16)
    false then false

copy-quarter is fn ((region : KernelByteRegion, destination : Nat, source : Nat)) -> Boolean
  copy-block (region, destination, source)
    true then copy-block (region, destination + 32, source + 32)
    false then false

clear-quarter is fn ((region : KernelByteRegion, destination : Nat)) -> Boolean
  clear-block (region, destination)
    true then clear-block (region, destination + 32)
    false then false

pub zero-quarter is fn ((region : KernelByteRegion, offset : Nat)) -> Boolean
  zero-block (region, offset)
    true then zero-block (region, offset + 32)
    false then false

copy-half is fn ((region : KernelByteRegion, destination : Nat, source : Nat)) -> Boolean
  copy-quarter (region, destination, source)
    true then copy-quarter (region, destination + 64, source + 64)
    false then false

clear-half is fn ((region : KernelByteRegion, destination : Nat)) -> Boolean
  clear-quarter (region, destination)
    true then clear-quarter (region, destination + 64)
    false then false

pub zero-half is fn ((region : KernelByteRegion, offset : Nat)) -> Boolean
  zero-quarter (region, offset)
    true then zero-quarter (region, offset + 64)
    false then false

pub copy-name is fn ((region : KernelByteRegion, destination : Nat, source : Nat)) -> Boolean
  copy-half (region, destination, source)
    true then copy-half (region, destination + 128, source + 128)
    false then false

pub clear-name is fn ((region : KernelByteRegion, destination : Nat)) -> Boolean
  clear-half (region, destination)
    true then clear-half (region, destination + 128)
    false then false

pub zero-name is fn ((region : KernelByteRegion, offset : Nat)) -> Boolean
  zero-half (region, offset)
    true then zero-half (region, offset + 128)
    false then false
