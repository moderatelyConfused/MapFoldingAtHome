const MAX_N: usize = 32;

#[derive(Copy, Clone)]
struct CacheAlignedArrays {
    big_p: [i32; MAX_N],
    c: [[i32; MAX_N]; MAX_N],
    d: [[[i32; MAX_N]; MAX_N]; MAX_N],
}

pub struct StampFolder {
    pub count: i64,
    cache: CacheAlignedArrays,
    a: [i32; MAX_N],
    b: [i32; MAX_N],
    count_array: [i32; MAX_N],
    gapter: [i32; MAX_N],
    gap: [i32; MAX_N * MAX_N],
}

impl Default for CacheAlignedArrays {
    fn default() -> Self {
        Self {
            big_p: [0; MAX_N],
            c: [[0; MAX_N]; MAX_N],
            d: [[[0; MAX_N]; MAX_N]; MAX_N],
        }
    }
}

impl StampFolder {
    #[inline(always)]
    pub fn new() -> Self {
        StampFolder {
            count: 0,
            cache: CacheAlignedArrays::default(),
            a: [0; MAX_N],
            b: [0; MAX_N],
            count_array: [0; MAX_N],
            gapter: [0; MAX_N],
            gap: [0; MAX_N * MAX_N],
        }
    }

    #[inline(always)]
    fn process(&mut self, n: i32) {
        self.count += n as i64;
    }

    #[inline(always)]
    fn calculate_big_p(&mut self, p: &[i32], dim: usize) {
        self.cache.big_p[0] = 1;
        for i in 1..=dim {
            self.cache.big_p[i] = self.cache.big_p[i - 1].wrapping_mul(p[i - 1]);
        }
    }

    #[inline(always)]
    fn calculate_c(&self, i: usize, m: i32, p: &[i32]) -> i32 {
        (m - 1) / self.cache.big_p[i - 1] - ((m - 1) / self.cache.big_p[i]) * p[i - 1] + 1
    }

    #[inline(always)]
    fn calculate_d(&self, i: usize, l: i32, m: i32, p: &[i32]) -> i32 {
        let l_idx = l as usize;
        let m_idx = m as usize;
        let delta = self.cache.c[i][l_idx] - self.cache.c[i][m_idx];
        
        if (delta & 1) == 0 {
            if self.cache.c[i][m_idx] == 1 { m } else { m - self.cache.big_p[i - 1] }
        } else if self.cache.c[i][m_idx] == p[i - 1] || m + self.cache.big_p[i - 1] > l {
            m
        } else {
            m + self.cache.big_p[i - 1]
        }
    }

    fn precalculate_arrays(&mut self, p: &[i32], n: i32, dim: usize) {
        self.calculate_big_p(p, dim);

        for i in 1..=dim {
            for m in 1..=n {
                let m_idx = m as usize;
                self.cache.c[i][m_idx] = self.calculate_c(i, m, p);
            }
        }

        for i in 1..=dim {
            for l in 1..=n {
                let l_idx = l as usize;
                for m in 1..=l {
                    let m_idx = m as usize;
                    self.cache.d[i][l_idx][m_idx] = self.calculate_d(i, l, m, p);
                }
            }
        }
    }

    #[inline(always)]
    fn process_gaps(&mut self, l: i32, g: &mut i32, gg: &mut i32, dd: &mut i32, dim: usize, res: i32, mod_val: i32) {
        for i in 1..=dim {
            if self.cache.d[i][l as usize][l as usize] == l {
                *dd += 1;
                continue;
            }
            
            let mut m = self.cache.d[i][l as usize][l as usize];
            while m != l {
                if mod_val == 0 || l != mod_val || m % mod_val == res {
                    self.gap[*gg as usize] = m;
                    self.count_array[m as usize] += 1;
                    if self.count_array[m as usize] == 1 {
                        *gg += 1;
                    }
                }
                m = self.cache.d[i][l as usize][self.b[m as usize] as usize];
            }
        }

        if *dd == dim as i32 {
            for m in 0..l {
                self.gap[*gg as usize] = m;
                *gg += 1;
            }
        }

        let g_start = *g;
        for j in g_start..*gg {
            let gap_j = self.gap[j as usize];
            self.gap[*g as usize] = gap_j;
            *g += (self.count_array[gap_j as usize] == (dim as i32 - *dd)) as i32;
            self.count_array[gap_j as usize] = 0;
        }
    }

    pub fn foldings(&mut self, p: &[i32], flag: bool, res: i32, mod_val: i32) {
        let n: i32 = p.iter().product();
        if n as usize >= MAX_N {
            panic!("Dimension too large");
        }

        let dim = p.len();
        self.precalculate_arrays(p, n, dim);

        let mut g = 0;
        let mut l = 1;

        while l > 0 {
            if !flag || l <= 1 || self.b[0] == 1 {
                if l > n {
                    self.process(n);
                } else {
                    let mut dd = 0;
                    let mut gg = self.gapter[(l - 1) as usize];
                    g = gg;
                    self.process_gaps(l, &mut g, &mut gg, &mut dd, dim, res, mod_val);
                }
            }

            while l > 0 && g == self.gapter[(l - 1) as usize] {
                l -= 1;
                if l > 0 {
                    let a_l = self.a[l as usize];
                    let b_l = self.b[l as usize];
                    self.b[a_l as usize] = b_l;
                    self.a[b_l as usize] = a_l;
                }
            }

            if l > 0 {
                g -= 1;
                let gap_g = self.gap[g as usize];
                self.a[l as usize] = gap_g;
                let b_gap = self.b[gap_g as usize];
                self.b[l as usize] = b_gap;
                self.b[gap_g as usize] = l;
                self.a[b_gap as usize] = l;
                self.gapter[l as usize] = g;
                l += 1;
            }
        }
    }

    #[cfg(test)]
    pub fn calculate_sequence(dimensions: &[i32]) -> i64 {
        if dimensions.iter().any(|&d| d == 0) {
            return 1;
        }

        let mut folder = StampFolder::new();
        folder.count = 0;
        folder.foldings(dimensions, true, 0, 0);
        folder.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_n_2() {
        let expected = vec![
            1, 2, 8, 60, 320, 1980, 10512, 60788, 320896,
            1787904, 9381840, 51081844
        ];

        for (i, &expected_value) in expected.iter().enumerate() {
            let dimensions = vec![i as i32, 2];
            let result = StampFolder::calculate_sequence(&dimensions);
            assert_eq!(
                result,
                expected_value,
                "Failed for n={}, width=2: expected {}, got {}",
                i,
                expected_value,
                result
            );
        }
    }

    #[test]
    fn test_sequence_n_3() {
        let expected = vec![
            1, 6, 60, 1368, 15552, 201240, 2016432, 21582624
        ];

        for (i, &expected_value) in expected.iter().enumerate() {
            let dimensions = vec![i as i32, 3];
            let result = StampFolder::calculate_sequence(&dimensions);
            assert_eq!(
                result,
                expected_value,
                "Failed for n={}, width=3: expected {}, got {}",
                i,
                expected_value,
                result
            );
        }
    }

    #[test]
    fn test_sequence_n_n() {
        let expected = vec![1, 1, 8, 1368, 300608];

        for (i, &expected_value) in expected.iter().enumerate() {
            let n = i as i32;
            let dimensions = vec![n, n];
            let result = StampFolder::calculate_sequence(&dimensions);
            assert_eq!(
                result,
                expected_value,
                "Failed for n×n where n={}: expected {}, got {}",
                n,
                expected_value,
                result
            );
        }
    }
}
