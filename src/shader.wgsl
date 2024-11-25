struct Params {
    @align(4) dim: u32,
    @align(4) n: u32,
    @align(4) mod_val: u32,
    @align(4) res: i32,
}

const MAX_N: u32 = 64u;
const MAX_N_SQ: u32 = 4096u;    // MAX_N * MAX_N

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> big_p: array<i32>;
@group(0) @binding(2) var<storage, read> c: array<array<i32, MAX_N>, MAX_N>;
@group(0) @binding(3) var<storage, read> d: array<i32>;
@group(0) @binding(4) var<storage, read_write> count_buffer: array<atomic<i32>>;

var<workgroup> gap: array<i32, MAX_N_SQ>;
var<workgroup> count_array: array<atomic<i32>, MAX_N>;
var<workgroup> gapter: array<i32, MAX_N>;
var<workgroup> a: array<i32, MAX_N>;
var<workgroup> b: array<i32, MAX_N>;

fn get_d_index(i: u32, l: u32, m: u32) -> u32 {
    return i * MAX_N * MAX_N + l * MAX_N + m;
}

fn process_gaps(l: i32, g: ptr<function, i32>, gg: ptr<function, i32>, dd: ptr<function, i32>) {
    for(var i: u32 = 1u; i <= params.dim; i = i + 1u) {
        let l_idx = u32(l);
        let d_idx = get_d_index(i, l_idx, l_idx);

        if (d[d_idx] == l) {
            *dd = *dd + 1;
            continue;
        }

        var m = d[d_idx];
        while (m != l) {
            if (params.mod_val == 0u || l != i32(params.mod_val) || m % i32(params.mod_val) == params.res) {
                gap[u32(*gg)] = m;
                atomicAdd(&count_array[u32(m)], 1);
                if (atomicLoad(&count_array[u32(m)]) == 1) {
                    *gg = *gg + 1;
                }
            }
            let new_idx = get_d_index(i, l_idx, u32(b[u32(m)]));
            m = d[new_idx];
        }
    }

    if (*dd == i32(params.dim)) {
        for(var m: i32 = 0; m < l; m = m + 1) {
            gap[u32(*gg)] = m;
            *gg = *gg + 1;
        }
    }

    let g_start = *g;
    for(var j: i32 = g_start; j < *gg; j = j + 1) {
        let gap_j = gap[u32(j)];
        gap[u32(*g)] = gap_j;
        if (atomicLoad(&count_array[u32(gap_j)]) == (i32(params.dim) - *dd)) {
            *g = *g + 1;
        }
        atomicStore(&count_array[u32(gap_j)], 0);
    }
}

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= params.n) {
        return;
    }

    // Initialize arrays
    for(var i: u32 = 0u; i < MAX_N; i = i + 1u) {
        gapter[i] = 0;
        a[i] = i32(i);
        b[i] = i32(i);
    }

    var g: i32 = 0;
    var l: i32 = 1;
    var local_count: i32 = 0;

    while (l > 0) {
        if (l <= 1 || b[0] == 1) {
            if (l > i32(params.n)) {
                local_count += i32(params.n);
            } else {
                var dd: i32 = 0;
                var gg: i32 = gapter[u32(l - 1)];
                g = gg;
                process_gaps(l, &g, &gg, &dd);
            }
        }

        while (l > 0 && g == gapter[u32(l - 1)]) {
            l = l - 1;
            if (l > 0) {
                let a_l = a[u32(l)];
                let b_l = b[u32(l)];
                b[u32(a_l)] = b_l;
                a[u32(b_l)] = a_l;
            }
        }

        if (l > 0) {
            g = g - 1;
            let gap_g = gap[u32(g)];
            a[u32(l)] = gap_g;
            let b_gap = b[u32(gap_g)];
            b[u32(l)] = b_gap;
            b[u32(gap_g)] = l;
            a[u32(b_gap)] = l;
            gapter[u32(l)] = g;
            l = l + 1;
        }
    }

    atomicAdd(&count_buffer[0], local_count);
}
