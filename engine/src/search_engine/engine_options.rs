use crate::create_options;

mod macros;

create_options! {
    EngineOptions {
        Options {
            //====== General ======
            ["Hash"]         hash:          i64   =>  32,  1,  524288;
            ["Threads"]      threads:       i64   =>  1,   1,  1024;
            ["MoveOverhead"] move_overhead: i64   =>  25,  0,  2000;
            ["MultiPV"]      multi_pv:      i64   =>  1,   1,  218;
            ["UCI_Chess960"] chess960:      bool  =>  false;
            ["UCI_ShowWDL"]  show_wdl:      bool  =>  false;

            //======== EAS ========
            ["UCI_Opponent"]  uci_opponent:   String  =>  String::from("");
            ["UCI_RatingAdv"] uci_rating_adv: i64     =>  -1000,  -5000,  5000;
            ["Contempt"]      min_contempt:   i64     =>  300,    -1000,  1000;
            ["DrawScore"]     draw_score:     i64     =>  30,     -100,   100;
            ["PolicySac"]     policy_sac:     i64     =>  10,     -100,   100;

            //======= Debug =======
            ["MinimalPrint"] minimal_print:  bool  =>  false;
            ["ItersAsNodes"] iters_as_nodes: bool  =>  false;
        }
        Tunables {
            //PST
            root_pst: f64  =>  3.515,  0.5,  5.0,  0.30,  0.002;
            base_pst: f64  =>  1.275,  0.5,  2.0,  0.20,  0.002;

            //CPUCT
            start_cpuct:       f64  =>  0.65,  0.1,  5.0,  0.10,  0.002;
            end_cpuct:         f64  =>  0.285,  0.1,  1.0,  0.03,  0.002;
            cpuct_depth_decay: f64  =>  0.210,  0.1,  5.0,  0.05,  0.002;

            //Visit scaling
            cpuct_visit_scale: f64  =>  8200.0,  4096.0,  65536.0,  500.0,  0.002;

            //Virtual loss
            virtual_loss_weight: f64  =>  1.190,  1.0,  5.0,  0.50,  0.002;

            //Variance scaling
            cpuct_variance_scale:  f64  =>  0.560,  0.1,   50.0,  0.15,  0.002;
            cpuct_variance_weight: f64  =>  0.640,  0.01,  2.0,   0.10,  0.002;
            cpuct_var_warmup:      f64  =>  0.360,  0.01,  1.0,   0.05,  0.002;

            //Exploration scale
            exploration_tau: f64  =>  0.635,  0.01,  1.0,  0.08,  0.002;

            //Gini impurity
            gini_base:       f64  =>  0.590,  0.1,   1.0,  0.05,  0.002;
            gini_multiplier: f64  =>  2.355,  0.5,   5.0,  0.20,  0.002;
            gini_min:        f64  =>  0.995,  0.01,  5.0,  0.10,  0.002;
            gini_max:        f64  =>  2.100,  0.01,  5.0,  0.10,  0.002;

            //Progressive widening
            policy_percentage:       f64  =>  0.685,  0.1,   1.0,    0.10,  0.002;
            min_policy_actions:      i64  =>  7,       1,     32,     2,     0.002;
            initial_visit_threshold: i64  =>  10,      4,     1024,   3,     0.002;
            visit_increase_multi:    f64  =>  1.780,   1.0,   10.0,   0.2,   0.002;
            visit_increase_offset:   f64  =>  4.930,   1.0,   10.0,   0.2,   0.002;

            //Butterfly history
            butterfly_reduction_factor: i64 => 9640,     1,    65536,     1024,    0.002;
            butterfly_bonus_scale:      f64 => 16850.0,  1.0,  131072.0,  2048.0,  0.002;

            //Draw Scaling
            power_50mr:          f64  =>  5.800,   1.0,    10.0,  0.5,    0.002;
            cap_50mr:            f64  =>  0.295,   0.01,   1.0,   0.1,    0.002;
            depth_scaling_power: f64  =>  1.135,   1.0,    5.0,   0.2,    0.002;
            depth_scaling:       f64  =>  0.0041,  0.0001, 1.0,   0.001,  0.002;
            depth_scaling_cap:   f64  =>  0.181,   0.01,   1.0,   0.02,   0.002;

            //Material Scaling
            knight_value:       f64  =>  605.0,      150.0,  750.0,   20.0,  0.002;
            bishop_value:       f64  =>  535.0,      150.0,  750.0,   20.0,  0.002;
            rook_value:         f64  =>  615.0,      400.0,  1000.0,  40.0,  0.002;
            queen_value:        f64  =>  1685.0,     900.0,  2000.0,  50.0,  0.002;
            scale_start_pos:    f64  =>  0.670,      0.5,    1.5,     0.1,   0.002;
            scale_zero_mat:     f64  =>  0.145,      0.1,    1.0,     0.1,   0.002;
            material_power:     f64  =>  0.655,      0.1,    5.0,     0.1,   0.002;
            wl_dampening_power: f64  =>  28.750,     1.0,    50.0,    3.0,   0.002;

            //Policy Sac
            sac_pawn_value:   f64  =>  63.0,   50.0,  200.0,  10.0,  0.002;
            sac_knight_value: f64  =>  300.0,  50.0,  500.0,  15.0,  0.002;
            sac_bishop_value: f64  =>  380.0,  50.0,  500.0,  15.0,  0.002;
            sac_rook_value:   f64  =>  480.0,  50.0,  800.0,  20.0,  0.002;
            sac_queen_value:  f64  =>  820.0,  50.0,  1500.0, 25.0,  0.002;

            //Time Manager
            default_moves_to_go:    f64  =>  28.0,       10.0,  50.0,  5.0,     0.002;
            phase_power:            f64  =>  1.370,      0.01,  10.0,  0.4,     0.002;
            phase_scale:            f64  =>  0.910,      0.01,  1.0,   0.1,     0.002;
            soft_constant:          f64  =>  0.008,      0.001, 1.0,   0.001,   0.002;
            soft_constant_multi:    f64  =>  0.0004,     0.0,   1.0,   0.0001,  0.002;
            soft_constant_cap:      f64  =>  0.005,      0.001, 1.0,   0.001,   0.002;
            soft_scale:             f64  =>  0.001,      0.001, 1.0,   0.003,   0.002;
            soft_scale_offset:      f64  =>  2.580,      0.1,   10.0,  0.5,     0.002;
            soft_scale_cap:         f64  =>  0.285,      0.1,   1.0,   0.05,    0.002;
            hard_constant:          f64  =>  0.350,      0.1,   10.0,  0.5,     0.002;
            hard_constant_multi:    f64  =>  3.300,      0.1,   10.0,  0.5,     0.002;
            hard_constant_cap:      f64  =>  3.450,      0.1,   10.0,  0.5,     0.002;
            hard_ply_div:           f64  =>  8.700,      1.0,   50.0,  2.0,     0.002;
            hard_scale_cap:         f64  =>  3.400,      0.1,   10.0,  0.5,     0.002;
            tm_bonus_scale:         f64  =>  0.470,      0.01,  1.0,   0.1,     0.002;
            tm_bonus_move_factor:   f64  =>  12.20,      1.0,   50.0,  2.0,     0.002;
            tm_bonus_ply_div:       f64  =>  4.550,      1.0,   20.0,  1.0,     0.002;
            tm_bonus_power:         f64  =>  1.680,      0.1,   10.0,  0.2,     0.002;
            time_fraction:          f64  =>  0.830,      0.01,  1.0,   0.1,     0.002;
            visit_distr_threshold:  f64  =>  0.680,      0.2,   0.8,   0.08,    0.002;
            visit_penalty_scale:    f64  =>  0.520,      0.01,  2.0,   0.1,     0.002;
            visit_penalty_multi:    f64  =>  9.10,       1.0,   50.0,  2.0,     0.002;
            visit_reward_scale:     f64  =>  0.795,      0.01,  2.0,   0.1,     0.002;
            visit_reward_multi:     f64  =>  5.95,       1.0,   50.0,  2.0,     0.002;
            gap_threshold:          f64  =>  0.760,      0.01,  0.99,  0.05,    0.002;
            gap_penalty_scale:      f64  =>  0.330,      0.01,  2.0,   0.04,    0.002;
            gap_penalty_multi:      f64  =>  25.50,      1.0,   50.0,  2.0,     0.002;
            gap_reward_scale:       f64  =>  0.160,      0.01,  2.0,   0.03,    0.002;
            gap_reward_multi:       f64  =>  14.30,      1.0,   50.0,  2.0,     0.002;
            falling_eval_ema_alpha: f64  =>  0.300,      0.01,  0.99,  0.08,    0.002;
            falling_eval_multi:     f64  =>  5.850,      0.1,   10.0,  0.8,     0.002;
            falling_eval_power:     f64  =>  1.960,      1.0,   3.0,   0.2,     0.002;
            falling_reward_clamp:   f64  =>  0.230,      0.01,  0.99,  0.05,    0.002;
            falling_penalty_clamp:  f64  =>  0.810,      0.01,  0.99,  0.1,     0.002;
            instability_ema_alpha:  f64  =>  0.160,      0.01,  0.99,  0.05,    0.002;
            instability_multi:      f64  =>  0.128,      0.01,  1.0,   0.05,    0.002;
            instability_scale:      f64  =>  0.880,      0.01,  2.0,   0.1,     0.002;
            behind_multi:           f64  =>  0.636,      0.01,  1.0,   0.05,    0.002;
            behind_scale:           f64  =>  0.470,      0.01,  2.0,   0.1,     0.002;
          
            //Transposition Table
            hash_size: f64  =>  0.142,  0.01,  0.5,  0.004,  0.002;
        }
        Variables {
            contempt: i64 = 0;
            kld_min: f64 = 0.5;
        }
    }
}