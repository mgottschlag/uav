#![no_main]
#![no_std]

use cortex_m::peripheral::DWT;
use panic_semihosting as _;
use rtic::cyccnt::U32Ext as _;

use board::{Board, EnginePwm};

mod board;

#[rtic::app(device = crate::board::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        engines: Engines,
    }

    #[init(schedule = [calibration1])]
    fn init(mut ctx: init::Context) -> init::LateResources {
        // Initialize (enable) the monotonic timer (CYCCNT)
        ctx.core.DCB.enable_trace();
        // required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
        DWT::unlock();
        ctx.core.DWT.enable_cycle_counter();

        let board = Board::init(ctx.core, ctx.device);

        let engine_pwm = board.engines;

        ctx.schedule
            .calibration1(ctx.start + 48_000_000.cycles())
            .unwrap();

        init::LateResources {
            engines: Engines {
                engine_pwm,
                engine_speed: 0,
                current_engine: 0,
            },
        }
    }
    #[task(schedule = [calibration2], resources = [engines])]
    fn calibration1(mut ctx: calibration1::Context) {
        let engines = &mut ctx.resources.engines;
        let max_duty = engines.engine_pwm.get_max_duty();
        engines.engine_pwm.set_duty([max_duty / 20; 4]);
        ctx.schedule
            .calibration2(ctx.scheduled + (48_000_000 * 4).cycles())
            .unwrap();
    }

    #[task(schedule = [engine_test], resources = [engines])]
    fn calibration2(mut ctx: calibration2::Context) {
        let engines = &mut ctx.resources.engines;
        let max_duty = engines.engine_pwm.get_max_duty();
        engines.engine_pwm.set_duty([max_duty / 10; 4]);
        ctx.schedule
            .engine_test(ctx.scheduled + (48_000_000 * 4).cycles())
            .unwrap();
    }

    #[task(schedule = [engine_test], resources = [engines])]
    fn engine_test(mut ctx: engine_test::Context) {
        let engines = &mut ctx.resources.engines;
        if engines.engine_speed == 100 {
            engines.engine_speed = 0;
            engines.current_engine = (engines.current_engine + 1) & 3;
        } else {
            engines.engine_speed += 10;
        }
        let max_duty = engines.engine_pwm.get_max_duty() as u32;
        let mut duty = [0; 4];
        // We want between 1-2ms of each 50ms PWM period.
        duty[engines.current_engine] =
            (max_duty / 20 + max_duty * engines.engine_speed / 2000) as u16;
        engines.engine_pwm.set_duty(duty);
        ctx.schedule
            .engine_test(ctx.scheduled + 48_000_000.cycles())
            .unwrap();
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop()
        }
    }

    extern "C" {
        fn EXTI0();
    }
};

pub struct Engines {
    engine_pwm: board::EnginePwmType,
    engine_speed: u32,
    current_engine: usize,
}
