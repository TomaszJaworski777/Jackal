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
            ["Contempt"]  contempt:   i64  =>  1000,  -10000,  10000;
            ["DrawScore"] draw_score: i64  =>  30,    -100,    100;
            ["PolicySac"] policy_sac: i64  =>  10,    0,       100;

            //======= Debug =======
            ["MinimalPrint"] minimal_print:  bool  =>  false;
            ["ItersAsNodes"] iters_as_nodes: bool  =>  false;
        }
        Tunables {
            //PST
            root_pst: f64  =>  3.2905,  0.5,  5.0,  0.25,   0.002;
            base_pst: f64  =>  1.2364,  0.5,  2.0,  0.15,  0.002;

            //CPUCT
            start_cpuct:       f64  =>  1.2813,    0.1,  5.0,  0.05,     0.002;
            end_cpuct:         f64  =>  0.3265,    0.1,  1.0,  0.015,    0.002;
            cpuct_depth_decay: f64  =>  0.264101,  0.1,  5.0,  0.02641,  0.002;

            //Visit scaling
            cpuct_visit_scale: f64  =>  8000.00,  4096.0,  65536.0,  250.0,  0.002;

            //Variance scaling
            cpuct_variance_scale:  f64  =>  0.2,   0.1,  50.0,  0.02,   0.002;
            cpuct_variance_weight: f64  =>  0.85,  0.0,  2.0,   0.085,  0.002;
            cpuct_var_warmup:      f64  =>  0.5,   0.0,  1.0,   0.05,   0.002;

            //Exploration scale
            exploration_tau: f64  =>  0.51,  0.0,  1.0,  0.055,  0.002;

            //Gini impurity
            gini_base:       f64  =>  0.463,  0.1,  1.0,  0.025,  0.002;
            gini_multiplier: f64  =>  1.567,  0.5,  5.0,  0.1,    0.002;
            gini_min:        f64  =>  1.0,    0.0,  5.0,  0.075,  0.002;
            gini_max:        f64  =>  1.5,    0.0,  5.0,  0.075,  0.002;

            //Progressive widening
            policy_percentage:       f64  =>  0.6407,  0.1,  1.0,    0.05,  0.002;
            min_policy_actions:      i64  =>  3,       1,    32,     1,     0.002;
            initial_visit_threshold: i64  =>  9,       0,    1024,   1,     0.002;
            visit_increase_multi:    f64  =>  2.2623,  1.0,  10.0,   0.1,   0.002;
            visit_increase_offset:   f64  =>  4.1092,  1.0,  10.0,   0.1,   0.002;

            //Butterfly history
            butterfly_reduction_factor: i64 => 8192,   1,  65536,   819,   0.002;
            butterfly_bonus_scale:      i64 => 16384,  1,  131072,  1638,  0.002;

            //Draw Scaling
            power_50mr:          f64  =>  3.0,     1.0,  10.0,  0.3,     0.002;
            cap_50mr:            f64  =>  0.9,     0.0,  1.0,   0.08,    0.002;
            depth_scaling_power: f64  =>  1.0,     1.0,  5.0,   0.1,     0.002;
            depth_scaling:       f64  =>  0.0015,  0.0,  1.0,   0.0001,  0.002;
            depth_scaling_cap:   f64  =>  0.15,    0.0,  1.0,   0.01,    0.002;

            //Material Scaling
            knight_value:         f64  =>  400.0,   150.0,  750.0,   25.0,  0.002;
            bishop_value:         f64  =>  400.0,   150.0,  750.0,   25.0,  0.002;
            rook_value:           f64  =>  750.0,   400.0,  1000.0,  30.0,  0.002;
            queen_value:          f64  =>  1500.0,  900.0,  2000.0,  35.0,  0.002;
            material_offset:      f64  =>  600.0,   400.0,  1200.0,  40.0,  0.002;
            material_scale:       f64  =>  36.0,    16.0,    64.0,    3.0,  0.002; 
            material_bonus_scale: f64  =>  1230.0,  500.0,  1500.0,  64.0,  0.002;

            //Policy Sac
            sac_pawn_value:   f64  =>  100.0,  50.0,  200.0,  5.0,  0.002;
            sac_knight_value: f64  =>  300.0,  50.0,  200.0,  5.0,  0.002;
            sac_bishop_value: f64  =>  300.0,  50.0,  200.0,  5.0,  0.002;
            sac_rook_value:   f64  =>  500.0,  50.0,  200.0,  5.0,  0.002;
            sac_queen_value:  f64  =>  900.0,  50.0,  200.0,  5.0,  0.002;

            //Time Manager
            default_moves_to_go:    f64  =>  30.0,         10.0,  50.0,  3.0,      0.002;
            phase_power:            f64  =>  2.0,          0.0,   10.0,  0.2,      0.002;
            phase_scale:            f64  =>  1.0,          0.0,   1.0,   0.1,      0.002;
            soft_constant:          f64  =>  0.0048,       0.0,   1.0,   0.0005,   0.002;
            soft_constant_multi:    f64  =>  0.00032,      0.0,   1.0,   0.00003,  0.002;
            soft_constant_cap:      f64  =>  0.006,        0.0,   1.0,   0.0006,   0.002;
            soft_scale:             f64  =>  0.0125,       0.0,   1.0,   0.0012,   0.002;
            soft_scale_offset:      f64  =>  2.5,          0.0,   10.0,  0.25,     0.002;
            soft_scale_cap:         f64  =>  0.25,         0.0,   1.0,   0.025,    0.002;
            hard_constant:          f64  =>  3.39,         0.0,   10.0,  0.339,    0.002;
            hard_constant_multi:    f64  =>  3.01,         0.0,   10.0,  0.301,    0.002;
            hard_constant_cap:      f64  =>  2.93,         0.0,   10.0,  0.293,    0.002;
            hard_ply_div:           f64  =>  12.0,         0.0,   50.0,  1.2,      0.002;
            hard_scale_cap:         f64  =>  4.0,          0.0,   10.0,  0.4,      0.002;
            tm_bonus_scale:         f64  =>  0.5,          0.0,   1.0,   0.05,     0.002;
            tm_bonus_move_factor:   f64  =>  10.0,         0.0,   50.0,  1.0,      0.002;
            tm_bonus_ply_div:       f64  =>  6.0,          0.0,   20.0,  0.6,      0.002;
            tm_bonus_power:         f64  =>  1.2,          0.0,   10.0,  0.12,     0.002;
            time_fraction:          f64  =>  0.85,         0.0,   1.0,   0.085,    0.002;
            visit_distr_threshold:  f64  =>  0.677245,     0.0,   1.0,   0.07,     0.002;
            visit_penalty_scale:    f64  =>  0.671748,     0.0,   2.0,   0.07,     0.002;
            visit_penalty_multi:    f64  =>  12.014090,    1.0,   50.0,  1.2,      0.002;
            visit_reward_scale:     f64  =>  0.846959,     0.0,   2.0,   0.1,      0.002;
            visit_reward_multi:     f64  =>  11.763412,    1.0,   50.0,  1.2,      0.002;
            gap_threshold:          f64  =>  0.445921,     0.0,   1.0,   0.045,    0.002;
            gap_penalty_scale:      f64  =>  0.227990,     0.0,   2.0,   0.02,     0.002;
            gap_penalty_multi:      f64  =>  18.823099,    1.0,   50.0,  1.8,      0.002;
            gap_reward_scale:       f64  =>  0.132607,     0.0,   2.0,   0.013,    0.002;
            gap_reward_multi:       f64  =>  14.032407,    1.0,   50.0,  1.4,      0.002;
            falling_eval_ema_alpha: f64  =>  0.558354,     0.0,   1.0,   0.055,    0.002;
            falling_eval_multi:     f64  =>  4.658633,     0.0,   10.0,  4.65,     0.002;
            falling_eval_power:     f64  =>  1.898156,     1.0,   3.0,   0.189,    0.002;
            falling_reward_clamp:   f64  =>  0.309756,     0.0,   1.0,   0.03,     0.002;
            falling_penalty_clamp:  f64  =>  0.665333,     0.0,   1.0,   0.066,    0.002;
            instability_ema_alpha:  f64  =>  0.221846,     0.0,   1.0,   0.022,    0.002;
            instability_multi:      f64  =>  0.278716679,  0.0,   1.0,   0.028,    0.002;
            instability_scale:      f64  =>  0.683337,     0.0,   2.0,   0.07,     0.002;
            behind_multi:           f64  =>  0.3882863,    0.0,   1.0,   0.038,    0.002;
            behind_scale:           f64  =>  0.470591,     0.0,   2.0,   0.047,    0.002;
          
            //Transposition Table
            hash_size: f64  =>  0.04,  0.01,  0.5,  0.004,  0.002;

            //Contempt
            max_reasonable_s: f64  =>  2.0,   0.0,    100.0,  0.2,    0.002;
            book_exit_bias:   f64  =>  0.65,  0.0,    1.0,    0.065,  0.002;
            draw_rate_target: f64  =>  0.0,   0.0,    1.0,    0.01,   0.002;
            draw_rate_ref:    f64  =>  0.65,  0.0,    1.0,    0.065,  0.002;
            contempt_att:     f64  =>  1.0,   -10.0,  10.0,   0.1,    0.002;
        }
    }
}