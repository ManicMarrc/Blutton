use bevy_ecs::prelude::*;
use macroquad::prelude::*;

fn win_conf() -> Conf {
  Conf { window_title: "Blutton".to_string(), window_height: 800, ..Default::default() }
}

#[derive(StageLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AppStage {
  Update,
  Render,
}

#[derive(SystemLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Render {
  Ui,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AppState {
  InGame,
}

const BUTTON_RADIUS: f32 = 100.0;
const UPGRADE_BUTTON_SIZE: f32 = 150.0;

const CLICKER_TIMER_MIN: f32 = 3.0;

mod component {
  use super::*;

  #[derive(Component)]
  pub struct Position(pub Vec2);
  #[derive(Component)]
  pub struct Button;

  #[derive(Component)]
  pub struct ClickPowerUpgrade;
  #[derive(Component)]
  pub struct ClickerUpgrade;
  #[derive(Component)]
  pub struct ClickerPowerUpgrade;
  #[derive(Component)]
  pub struct ClickerTimerUpgrade;

  #[derive(Component)]
  pub struct Clicker {
    pub timer: f32,
  }
}
pub use component::*;

mod resource {
  use super::*;

  #[derive(Resource)]
  pub struct ClickCount(pub usize);
  #[derive(Resource)]
  pub struct ClickPower(pub usize);
  #[derive(Resource)]
  pub struct ClickerCount(pub usize);
  #[derive(Resource)]
  pub struct ClickerPower(pub usize);
  #[derive(Resource)]
  pub struct ClickerTimer(pub f32);

  #[derive(Resource)]
  pub struct ClickPowerUpgradeCost(pub usize);
  #[derive(Resource)]
  pub struct ClickerUpgradeCost(pub usize);
  #[derive(Resource)]
  pub struct ClickerPowerUpgradeCost(pub usize);
  #[derive(Resource)]
  pub struct ClickerTimerUpgradeCost(pub usize);
}
pub use resource::*;

#[macroquad::main(win_conf)]
async fn main() {
  let mut world = World::new();
  world.insert_resource(State::new(AppState::InGame));

  world.insert_resource(ClickCount(0));
  world.insert_resource(ClickPower(1));
  world.insert_resource(ClickerCount(0));
  world.insert_resource(ClickerPower(1));
  world.insert_resource(ClickerTimer(10.0));

  world.insert_resource(ClickPowerUpgradeCost(10));
  world.insert_resource(ClickerUpgradeCost(50));
  world.insert_resource(ClickerPowerUpgradeCost(250));
  world.insert_resource(ClickerTimerUpgradeCost(150));

  let mut schedule = Schedule::default();
  schedule.add_stage(
    AppStage::Update,
    SystemStage::parallel().with_system_set(State::<AppState>::get_driver()),
  );
  schedule.add_stage_after(
    AppStage::Update,
    AppStage::Render,
    SystemStage::single_threaded().with_system_set(State::<AppState>::get_driver()),
  );

  {
    use in_game::*;

    schedule.add_system_set_to_stage(
      AppStage::Update,
      SystemSet::on_enter(AppState::InGame)
        .with_system(update::button_setup)
        .with_system(update::upgrades_setup),
    );

    schedule.add_system_set_to_stage(
      AppStage::Update,
      SystemSet::on_update(AppState::InGame)
        .with_system(update::button_onclick)
        .with_system(update::click_power_upgrade_onclick)
        .with_system(update::clicker_upgrade_onclick)
        .with_system(update::clicker_power_upgrade_onclick)
        .with_system(update::clicker_timer_upgrade_onclick)
        .with_system(update::clicker_update)
        .with_system(update::button_pos_sync),
    );

    schedule.add_system_set_to_stage(
      AppStage::Render,
      SystemSet::on_update(AppState::InGame)
        .with_system(render::button_draw.before(Render::Ui))
        .with_system(render::click_power_upgrade_draw.label(Render::Ui))
        .with_system(render::clicker_upgrade_draw.label(Render::Ui))
        .with_system(render::clicker_power_upgrade_draw.label(Render::Ui))
        .with_system(render::clicker_timer_upgrade_draw.label(Render::Ui)),
    );
  };

  loop {
    clear_background(WHITE);

    schedule.run(&mut world);

    next_frame().await;
  }
}

mod in_game {
  use super::*;

  pub mod update {
    use super::*;

    pub fn button_setup(mut commands: Commands) {
      commands.spawn(Button).insert(Position(vec2(screen_width(), screen_height()) / 2.0));
    }

    pub fn upgrades_setup(mut commands: Commands) {
      commands.spawn(ClickPowerUpgrade).insert(Position(vec2(10.0, 10.0)));
      commands
        .spawn(ClickerUpgrade)
        .insert(Position(vec2(10.0, 10.0 + UPGRADE_BUTTON_SIZE + 10.0)));
      commands
        .spawn(ClickerPowerUpgrade)
        .insert(Position(vec2(10.0, 10.0 + (UPGRADE_BUTTON_SIZE + 10.0) * 2.0)));
      commands
        .spawn(ClickerTimerUpgrade)
        .insert(Position(vec2(10.0, 10.0 + (UPGRADE_BUTTON_SIZE + 10.0) * 3.0)));
    }

    pub fn button_onclick(
      buttons: Query<&Position, With<Button>>,
      mut click_count: ResMut<ClickCount>,
      click_power: Res<ClickPower>,
    ) {
      let (mouse_x, mouse_y) = mouse_position();
      let mouse_pos = vec2(mouse_x, mouse_y);

      for button in &buttons {
        if is_mouse_button_pressed(MouseButton::Left)
          && Circle::new(button.0.x, button.0.y, BUTTON_RADIUS).contains(&mouse_pos)
        {
          click_count.0 += click_power.0;
        }
      }
    }

    pub fn click_power_upgrade_onclick(
      click_power_upgrades: Query<&Position, With<ClickPowerUpgrade>>,
      mut click_count: ResMut<ClickCount>,
      mut click_power: ResMut<ClickPower>,
      mut click_power_cost: ResMut<ClickPowerUpgradeCost>,
    ) {
      let (mouse_x, mouse_y) = mouse_position();
      let mouse_pos = vec2(mouse_x, mouse_y);

      for click_power_upgrade in &click_power_upgrades {
        if click_count.0 >= click_power_cost.0
          && is_mouse_button_pressed(MouseButton::Left)
          && Rect::new(
            click_power_upgrade.0.x,
            click_power_upgrade.0.y,
            UPGRADE_BUTTON_SIZE,
            UPGRADE_BUTTON_SIZE,
          )
          .contains(mouse_pos)
        {
          click_count.0 -= click_power_cost.0;
          click_power.0 += 1;
          click_power_cost.0 = (click_power.0 as f64).powf(4.5).ceil() as usize;
        }
      }
    }

    pub fn clicker_upgrade_onclick(
      mut commands: Commands,
      clicker_upgrades: Query<&Position, With<ClickerUpgrade>>,
      mut click_count: ResMut<ClickCount>,
      mut clicker_count: ResMut<ClickerCount>,
      mut clicker_cost: ResMut<ClickerUpgradeCost>,
    ) {
      let (mouse_x, mouse_y) = mouse_position();
      let mouse_pos = vec2(mouse_x, mouse_y);

      for clicker_upgrade in &clicker_upgrades {
        if click_count.0 >= clicker_cost.0
          && is_mouse_button_pressed(MouseButton::Left)
          && Rect::new(
            clicker_upgrade.0.x,
            clicker_upgrade.0.y,
            UPGRADE_BUTTON_SIZE,
            UPGRADE_BUTTON_SIZE,
          )
          .contains(mouse_pos)
        {
          click_count.0 -= clicker_cost.0;
          clicker_count.0 += 1;
          clicker_cost.0 *= 2;
          commands.spawn(Clicker { timer: 0.0 });
        }
      }
    }

    pub fn clicker_power_upgrade_onclick(
      clicker_power_upgrades: Query<&Position, With<ClickerPowerUpgrade>>,
      mut click_count: ResMut<ClickCount>,
      mut clicker_power: ResMut<ClickerPower>,
      mut clicker_power_cost: ResMut<ClickerPowerUpgradeCost>,
    ) {
      let (mouse_x, mouse_y) = mouse_position();
      let mouse_pos = vec2(mouse_x, mouse_y);

      for clicker_power_upgrade in &clicker_power_upgrades {
        if click_count.0 >= clicker_power_cost.0
          && is_mouse_button_pressed(MouseButton::Left)
          && Rect::new(
            clicker_power_upgrade.0.x,
            clicker_power_upgrade.0.y,
            UPGRADE_BUTTON_SIZE,
            UPGRADE_BUTTON_SIZE,
          )
          .contains(mouse_pos)
        {
          click_count.0 -= clicker_power_cost.0;
          clicker_power.0 += 1;
          clicker_power_cost.0 *= 2;
        }
      }
    }

    pub fn clicker_timer_upgrade_onclick(
      clicker_timer_upgrades: Query<&Position, With<ClickerTimerUpgrade>>,
      mut click_count: ResMut<ClickCount>,
      mut clicker_timer: ResMut<ClickerTimer>,
      mut clicker_timer_cost: ResMut<ClickerTimerUpgradeCost>,
    ) {
      if clicker_timer.0 > CLICKER_TIMER_MIN {
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_pos = vec2(mouse_x, mouse_y);

        for clicker_timer_upgrade in &clicker_timer_upgrades {
          if click_count.0 >= clicker_timer_cost.0
            && is_mouse_button_pressed(MouseButton::Left)
            && Rect::new(
              clicker_timer_upgrade.0.x,
              clicker_timer_upgrade.0.y,
              UPGRADE_BUTTON_SIZE,
              UPGRADE_BUTTON_SIZE,
            )
            .contains(mouse_pos)
          {
            click_count.0 -= clicker_timer_cost.0;
            clicker_timer.0 -= 0.5;
            clicker_timer_cost.0 *= 2;
          }
        }
      }
    }

    pub fn clicker_update(
      mut clickers: Query<&mut Clicker>,
      mut click_count: ResMut<ClickCount>,
      clicker_power: Res<ClickerPower>,
      clicker_timer: Res<ClickerTimer>,
    ) {
      for mut clicker in &mut clickers {
        clicker.timer += get_frame_time();

        if clicker.timer >= clicker_timer.0 {
          clicker.timer = 0.0;
          click_count.0 += clicker_power.0;
        }
      }
    }

    pub fn button_pos_sync(mut buttons: Query<&mut Position, With<Button>>) {
      for mut button in &mut buttons {
        button.0 = vec2(screen_width(), screen_height()) / 2.0;
      }
    }
  }

  pub mod render {
    use super::*;

    pub fn button_draw(buttons: Query<&Position, With<Button>>, click_count: Res<ClickCount>) {
      let text = click_count.0.to_string();
      let text_measure = measure_text(&text, None, 64, 1.0);

      for button in &buttons {
        draw_circle(button.0.x, button.0.y, BUTTON_RADIUS, RED);

        draw_text(
          &text,
          button.0.x - text_measure.width / 2.0,
          button.0.y - text_measure.height / 2.0 + text_measure.offset_y,
          64.0,
          WHITE,
        );
      }
    }

    pub fn click_power_upgrade_draw(
      click_power_upgrades: Query<&Position, With<ClickPowerUpgrade>>,
      click_power: Res<ClickPower>,
      click_power_cost: Res<ClickPowerUpgradeCost>,
    ) {
      let name = "Power";
      let name_measure = measure_text(name, None, 32, 1.0);

      let click_power = click_power.0.to_string();
      let click_power_measure = measure_text(&click_power, None, 32, 1.0);

      let click_power_cost = click_power_cost.0.to_string();
      let click_power_cost_measure = measure_text(&click_power_cost, None, 32, 1.0);

      for click_power_upgrade in &click_power_upgrades {
        draw_rectangle(
          click_power_upgrade.0.x,
          click_power_upgrade.0.y,
          UPGRADE_BUTTON_SIZE,
          UPGRADE_BUTTON_SIZE,
          RED,
        );

        draw_text(
          name,
          click_power_upgrade.0.x + UPGRADE_BUTTON_SIZE / 2.0 - name_measure.width / 2.0,
          click_power_upgrade.0.y + name_measure.height / 2.0 + name_measure.offset_y,
          32.0,
          WHITE,
        );

        draw_text(
          &click_power,
          click_power_upgrade.0.x + UPGRADE_BUTTON_SIZE / 2.0 - click_power_measure.width / 2.0,
          click_power_upgrade.0.y + UPGRADE_BUTTON_SIZE / 2.0 - click_power_measure.height / 2.0
            + click_power_measure.offset_y
            - click_power_cost_measure.offset_y / 2.0
            + 10.0
            + name_measure.height / 2.0,
          32.0,
          WHITE,
        );
        draw_text(
          &click_power_cost,
          click_power_upgrade.0.x + UPGRADE_BUTTON_SIZE / 2.0
            - click_power_cost_measure.width / 2.0,
          click_power_upgrade.0.y + UPGRADE_BUTTON_SIZE / 2.0 - click_power_measure.height / 2.0
            + click_power_measure.offset_y
            + click_power_cost_measure.offset_y / 2.0
            + 20.0
            + name_measure.height / 2.0,
          32.0,
          WHITE,
        );
      }
    }

    pub fn clicker_upgrade_draw(
      clicker_upgrades: Query<&Position, With<ClickerUpgrade>>,
      clicker_count: Res<ClickerCount>,
      clicker_cost: Res<ClickerUpgradeCost>,
    ) {
      let name = "Clicker";
      let name_measure = measure_text(name, None, 32, 1.0);

      let clicker = clicker_count.0.to_string();
      let clicker_measure = measure_text(&clicker, None, 32, 1.0);

      let clicker_cost = clicker_cost.0.to_string();
      let clicker_cost_measure = measure_text(&clicker_cost, None, 32, 1.0);

      for clicker_upgrade in &clicker_upgrades {
        draw_rectangle(
          clicker_upgrade.0.x,
          clicker_upgrade.0.y,
          UPGRADE_BUTTON_SIZE,
          UPGRADE_BUTTON_SIZE,
          RED,
        );

        draw_text(
          name,
          clicker_upgrade.0.x + UPGRADE_BUTTON_SIZE / 2.0 - name_measure.width / 2.0,
          clicker_upgrade.0.y + name_measure.height / 2.0 + name_measure.offset_y,
          32.0,
          WHITE,
        );

        draw_text(
          &clicker,
          clicker_upgrade.0.x + UPGRADE_BUTTON_SIZE / 2.0 - clicker_measure.width / 2.0,
          clicker_upgrade.0.y + UPGRADE_BUTTON_SIZE / 2.0 - clicker_measure.height / 2.0
            + clicker_measure.offset_y
            - clicker_cost_measure.offset_y / 2.0
            + 10.0
            + name_measure.height / 2.0,
          32.0,
          WHITE,
        );
        draw_text(
          &clicker_cost,
          clicker_upgrade.0.x + UPGRADE_BUTTON_SIZE / 2.0 - clicker_cost_measure.width / 2.0,
          clicker_upgrade.0.y + UPGRADE_BUTTON_SIZE / 2.0 - clicker_measure.height / 2.0
            + clicker_measure.offset_y
            + clicker_cost_measure.offset_y / 2.0
            + 20.0
            + name_measure.height / 2.0,
          32.0,
          WHITE,
        );
      }
    }

    pub fn clicker_power_upgrade_draw(
      clicker_power_upgrades: Query<&Position, With<ClickerPowerUpgrade>>,
      clicker_power: Res<ClickerPower>,
      clicker_power_cost: Res<ClickerPowerUpgradeCost>,
    ) {
      let name = "ClickerPow";
      let name_measure = measure_text(name, None, 32, 1.0);

      let clicker_power = clicker_power.0.to_string();
      let clicker_power_measure = measure_text(&clicker_power, None, 32, 1.0);

      let clicker_power_cost = clicker_power_cost.0.to_string();
      let clicker_power_cost_measure = measure_text(&clicker_power_cost, None, 32, 1.0);

      for clicker_power_upgrade in &clicker_power_upgrades {
        draw_rectangle(
          clicker_power_upgrade.0.x,
          clicker_power_upgrade.0.y,
          UPGRADE_BUTTON_SIZE,
          UPGRADE_BUTTON_SIZE,
          RED,
        );

        draw_text(
          name,
          clicker_power_upgrade.0.x + UPGRADE_BUTTON_SIZE / 2.0 - name_measure.width / 2.0,
          clicker_power_upgrade.0.y + name_measure.height / 2.0 + name_measure.offset_y,
          32.0,
          WHITE,
        );

        draw_text(
          &clicker_power,
          clicker_power_upgrade.0.x + UPGRADE_BUTTON_SIZE / 2.0 - clicker_power_measure.width / 2.0,
          clicker_power_upgrade.0.y + UPGRADE_BUTTON_SIZE / 2.0
            - clicker_power_measure.height / 2.0
            + clicker_power_measure.offset_y
            - clicker_power_cost_measure.offset_y / 2.0
            + 10.0
            + name_measure.height / 2.0,
          32.0,
          WHITE,
        );
        draw_text(
          &clicker_power_cost,
          clicker_power_upgrade.0.x + UPGRADE_BUTTON_SIZE / 2.0
            - clicker_power_cost_measure.width / 2.0,
          clicker_power_upgrade.0.y + UPGRADE_BUTTON_SIZE / 2.0
            - clicker_power_measure.height / 2.0
            + clicker_power_measure.offset_y
            + clicker_power_cost_measure.offset_y / 2.0
            + 20.0
            + name_measure.height / 2.0,
          32.0,
          WHITE,
        );
      }
    }

    pub fn clicker_timer_upgrade_draw(
      clicker_timer_upgrades: Query<&Position, With<ClickerTimerUpgrade>>,
      clicker_timer: Res<ClickerTimer>,
      clicker_timer_cost: Res<ClickerTimerUpgradeCost>,
    ) {
      let name = "ClickerTim";
      let name_measure = measure_text(name, None, 32, 1.0);

      let clicker_timer_text = clicker_timer.0.to_string();
      let clicker_timer_measure = measure_text(&clicker_timer_text, None, 32, 1.0);

      let clicker_timer_cost = if clicker_timer.0 <= CLICKER_TIMER_MIN {
        "MAX".to_string()
      } else {
        clicker_timer_cost.0.to_string()
      };
      let clicker_timer_cost_measure = measure_text(&clicker_timer_cost, None, 32, 1.0);

      for clicker_timer_upgrade in &clicker_timer_upgrades {
        draw_rectangle(
          clicker_timer_upgrade.0.x,
          clicker_timer_upgrade.0.y,
          UPGRADE_BUTTON_SIZE,
          UPGRADE_BUTTON_SIZE,
          RED,
        );

        draw_text(
          name,
          clicker_timer_upgrade.0.x + UPGRADE_BUTTON_SIZE / 2.0 - name_measure.width / 2.0,
          clicker_timer_upgrade.0.y + name_measure.height / 2.0 + name_measure.offset_y,
          32.0,
          WHITE,
        );

        draw_text(
          &clicker_timer_text,
          clicker_timer_upgrade.0.x + UPGRADE_BUTTON_SIZE / 2.0 - clicker_timer_measure.width / 2.0,
          clicker_timer_upgrade.0.y + UPGRADE_BUTTON_SIZE / 2.0
            - clicker_timer_measure.height / 2.0
            + clicker_timer_measure.offset_y
            - clicker_timer_cost_measure.offset_y / 2.0
            + 10.0
            + name_measure.height / 2.0,
          32.0,
          WHITE,
        );
        draw_text(
          &clicker_timer_cost,
          clicker_timer_upgrade.0.x + UPGRADE_BUTTON_SIZE / 2.0
            - clicker_timer_cost_measure.width / 2.0,
          clicker_timer_upgrade.0.y + UPGRADE_BUTTON_SIZE / 2.0
            - clicker_timer_measure.height / 2.0
            + clicker_timer_measure.offset_y
            + clicker_timer_cost_measure.offset_y / 2.0
            + 20.0
            + name_measure.height / 2.0,
          32.0,
          WHITE,
        );
      }
    }
  }
}
