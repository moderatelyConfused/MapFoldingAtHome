pub struct StampFolder {
    count: i64,
}

impl StampFolder {
    pub fn new() -> Self {
        StampFolder {
            count: 0,
        }
    }

    pub fn process(&mut self, _a: &[i32], _b: &[i32], n: i32) {
        self.count += n as i64;
    }

    pub fn foldings(&mut self, p: &[i32], flag: bool, res: i32, mod_val: i32) {
        // Calculate total number of leaves
        let n: i32 = p.iter().product();

        // Initialize arrays
        let mut a = vec![0; (n + 1) as usize];
        let mut b = vec![0; (n + 1) as usize];
        let mut count = vec![0; (n + 1) as usize];
        let mut gapter = vec![0; (n + 1) as usize];
        let mut gap = vec![0; (n * n + 1) as usize];

        let dim = p.len();
        let mut big_p = vec![1; dim + 1];
        let mut c = vec![vec![0; (n + 1) as usize]; dim + 1];
        let mut d = vec![vec![vec![0; (n + 1) as usize]; (n + 1) as usize]; dim + 1];

        // Calculate big_p array
        for i in 1..=dim {
            big_p[i] = big_p[i - 1] * p[i - 1];
        }

        // Calculate c array
        for i in 1..=dim {
            for m in 1..=n {
                c[i][m as usize] = (m - 1) / big_p[i - 1] - ((m - 1) / big_p[i]) * p[i - 1] + 1;
            }
        }

        // Calculate d array
        for i in 1..=dim {
            for l in 1..=n {
                for m in 1..=l {
                    let delta = c[i][l as usize] - c[i][m as usize];
                    d[i][l as usize][m as usize] = if (delta & 1) == 0 {
                        if c[i][m as usize] == 1 {
                            m
                        } else {
                            m - big_p[i - 1]
                        }
                    } else if c[i][m as usize] == p[i - 1] || m + big_p[i - 1] > l {
                        m
                    } else {
                        m + big_p[i - 1]
                    };
                }
            }
        }

        let mut g = 0;
        let mut l = 1;

        // Main backtrack loop
        while l > 0 {
            if !flag || l <= 1 || b[0] == 1 {
                if l > n {
                    self.process(&a, &b, n);
                } else {
                    let mut dd = 0;
                    let mut gg = gapter[(l - 1) as usize];
                    g = gg;

                    // Append potential gaps
                    for i in 1..=dim {
                        if d[i][l as usize][l as usize] == l {
                            dd += 1;
                        } else {
                            let mut m = d[i][l as usize][l as usize];
                            while m != l {
                                if mod_val == 0 || l != mod_val || m % mod_val == res {
                                    gap[gg as usize] = m;
                                    count[m as usize] += 1;
                                    if count[m as usize] == 1 {
                                        gg += 1;
                                    }
                                }
                                m = d[i][l as usize][b[m as usize] as usize];
                            }
                        }
                    }

                    // Handle case when dd == dim
                    if dd == dim {
                        for m in 0..l {
                            gap[gg as usize] = m;
                            gg += 1;
                        }
                    }

                    // Process gaps
                    for j in g..gg {
                        gap[g as usize] = gap[j as usize];
                        if count[gap[j as usize] as usize] == (dim - dd) as i32 {
                            g += 1;
                        }
                        count[gap[j as usize] as usize] = 0;
                    }
                }
            }

            // Backtrack if necessary
            while l > 0 && g == gapter[(l - 1) as usize] {
                l -= 1;
                if l > 0 {
                    b[a[l as usize] as usize] = b[l as usize];
                    a[b[l as usize] as usize] = a[l as usize];
                }
            }

            // Insert new leaf and advance
            if l > 0 {
                g -= 1;
                a[l as usize] = gap[g as usize];
                b[l as usize] = b[a[l as usize] as usize];
                b[a[l as usize] as usize] = l;
                a[b[l as usize] as usize] = l;
                gapter[l as usize] = g;
                l += 1;
            }
        }
    }

    #[cfg(test)]
    pub fn calculate_sequence(dimensions: &[i32]) -> i64 {
        // Special case: if any dimension is 0, return 1
        if dimensions.iter().any(|&d| d == 0) {
            return 1;
        }

        let mut folder = StampFolder::new();
        folder.count = 0;
        folder.foldings(dimensions, true, 0, 0);
        folder.count
    }
}
