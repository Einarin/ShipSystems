//use std::option::Option::*;
//use std::cmp::Ordering;
use std::ops::{Add,AddAssign};
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use std::fmt::Display;
//#[macro_use]
//extern crate bitflags;

#[derive(PartialEq,Copy,Clone,Debug)]
pub struct Resources {
    electricity: f32,
    heat: f32,
    deuterium: f32,
}

impl Resources {
    fn electric(amount: f32) -> Resources {
        Resources{ electricity: amount, heat: 0.0, deuterium: 0.0 }
    }
}

impl Display for Resources {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}W {}J {}g",self.electricity, self.heat, self.deuterium)
    }
}

impl Default for Resources {
    fn default() -> Resources {
        Resources { electricity: 0.0, heat: 0.0, deuterium: 0.0 }
    }
}

impl Add for Resources {
    type Output = Resources;
    fn add(self, rhs: Resources) -> Resources {
        Resources {
            electricity: self.electricity + rhs.electricity,
            heat: self.heat + rhs.heat,
            deuterium: self.deuterium + rhs.deuterium,
        }
    }
}

impl AddAssign for Resources {
    fn add_assign(&mut self, rhs: Resources) {
        self.electricity += rhs.electricity;
        self.heat += rhs.heat;
        self.deuterium += rhs.deuterium;
    }
}

pub struct GameTime {
    ms: u64,
}

/*bitflags! {
    pub flags ComponentPhase: u32 {
        const STANDARD      = 0b001,
        const REGULATION    = 0b010,
        const OVERFLOW      = 0b100,
    }
}*/

pub trait Component : Display {
    fn get_fixed_processing(&mut self, time: &GameTime) -> Resources {
        Resources::default()
    }
    fn get_potential_supply(&self, time: &GameTime) -> Resources {
        Resources::default()
    }
    fn get_potential_consumption(&self, resources: &Resources, time: &GameTime) -> Resources {
        Resources::default()
    }
    fn supply_on_demand(&mut self, resources: &Resources, time: &GameTime) -> Resources {
        Resources::default()
    }
    fn consume_on_demand(&mut self, resources: &Resources, time: &GameTime) -> Resources {
        Resources::default()
    }
}


enum ReactorPhase {
    Running,
    Stopped
}
pub struct FusionReactor {
    phase: ReactorPhase,
    sizing: f32,
    last_utilization: f32,
}

impl FusionReactor {
    fn compute_load_level(&mut self, load: f32, time: &GameTime) -> Resources{
        self.last_utilization = load;
        Resources {
            electricity: load * self.sizing * time.ms as f32,
            heat: load * 5.0 * self.sizing * time.ms as f32,
            deuterium: load * -0.001 * self.sizing * time.ms as f32,
        }
    }
    fn compute_demand_level(&mut self, demand: f32, time: &GameTime) -> Resources{
        let load = min(1.0,demand / self.sizing * time.ms as f32);
        self.compute_load_level(load,time)
    }
    fn new_component(size: f32) -> Rc<RefCell<Box<Component>>> {
        Rc::new(RefCell::new(Box::new(FusionReactor{phase: ReactorPhase::Stopped, sizing: size, last_utilization: 0.0}) as Box<Component>))
    }
}

impl Display for FusionReactor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.phase {
            ReactorPhase::Stopped => write!(f, "Reactor stopped."),
            ReactorPhase::Running => {
                if self.last_utilization == -1.0 {
                    write!(f, "Reactor started!")
                } else {
                    write!(f, "Reactor running at {}%", self.last_utilization * 100.0)
                }
            }
        }
    }
}

impl Component for FusionReactor {
    fn get_fixed_processing(&mut self, time: &GameTime) -> Resources {
        match self.phase {
            ReactorPhase::Running => self.compute_load_level(0.01,time),
            ReactorPhase::Stopped => Resources::default()
        }
    }
    fn get_potential_supply(&self, time: &GameTime) -> Resources {
        match self.phase {
            ReactorPhase::Running => Resources {
                electricity: 0.2 * self.sizing,
                heat: 5.0 * self.sizing,
                deuterium: 0.0
            },
            ReactorPhase::Stopped => Resources::default(),
        }
    }
    fn get_potential_consumption(&self, resources: &Resources, time: &GameTime) -> Resources {
        match self.phase {
            ReactorPhase::Running => Resources {
                electricity: 0.0,
                heat: 5.0 * self.sizing,
                deuterium: min(resources.deuterium,-0.001 * self.sizing),
            },
            ReactorPhase::Stopped => Resources {
                electricity: min(resources.electricity,-0.2 * self.sizing),
                heat: 10.0 * self.sizing,
                deuterium : min(resources.deuterium,-0.1 * self.sizing),
            },
        }
    }
    fn consume_on_demand(&mut self, resources: &Resources, time: &GameTime) -> Resources {
        match self.phase {
            ReactorPhase::Running => Resources::default(),
            ReactorPhase::Stopped => {
                if resources.electricity >= (0.2 * self.sizing) {
                    self.phase = ReactorPhase::Running;
                    self.last_utilization = -1.0;
                    Resources{
                        electricity: -0.2 * self.sizing,
                        heat: 10.0 * self.sizing,
                        deuterium : -0.1 * self.sizing,
                    }
                } else {
                    Resources::default()
                }
            },
        }
    }
    fn supply_on_demand(&mut self, resources: &Resources, time: &GameTime) -> Resources {
        match self.phase {
            ReactorPhase::Running => {
                if resources.electricity < 0.0 {
                    self.compute_demand_level(-resources.electricity * 0.99,time)
                } else {
                    Resources::default()
                }
            },
            ReactorPhase::Stopped => Resources::default(),
        }
    }
}

pub struct Capacitor {
    capacity: f32,
    charge_level: f32,
}

impl Capacitor {
    fn new_component(capacity: f32) -> Rc<RefCell<Box<Component>>> {
        Rc::new(RefCell::new(Box::new(Capacitor{capacity: capacity, charge_level: 0.0}) as Box<Component>))
    }
}

impl Component for Capacitor {
    fn get_potential_supply(&self, time: &GameTime) -> Resources {
        Resources::electric(self.charge_level)
    }
    fn get_potential_consumption(&self, resources: &Resources, time: &GameTime) -> Resources {
        let mut output = Resources::default();
        output.electricity -= self.capacity - self.charge_level;
        output
    }
    fn supply_on_demand(&mut self, resources: &Resources, time: &GameTime) -> Resources {
        let mut output = Resources::default();
        if resources.electricity < 0.0 {
            let delta = min(self.charge_level,-resources.electricity * time.ms as f32);
            self.charge_level -= delta;
            output.electricity += delta;
        }
        output
    }
    fn consume_on_demand(&mut self, resources: &Resources, time: &GameTime) -> Resources {
        let mut output = Resources::default();
        if resources.electricity > 0.0 {
            let delta = min(self.capacity - self.charge_level,resources.electricity * time.ms as f32);
            self.charge_level += delta;
            output.electricity -= delta;
        }
        output
    }
}

impl Display for Capacitor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "capacitor at {}% charge", 100.0 * self.charge_level / self.capacity)
    }
}

pub struct BatteryData {
    capacity: f32,
    charge_rate: f32,
    discharge_rate: f32,
    charge_level: f32,
}

pub struct Battery {
    data: Rc<RefCell<BatteryData>>,
}

impl BatteryData {
    pub fn new(capacity: f32, charge: f32) -> Rc<RefCell<BatteryData>> {
        Rc::new(RefCell::new(BatteryData{
            capacity: capacity,
            charge_rate: 0.01 * capacity,
            discharge_rate: 0.05 * capacity,
            charge_level: charge * capacity,
        }))
    }

    pub fn is_charged(&self) -> bool {
        self.charge_level >= self.capacity
    }
}

impl Display for Battery {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.data.borrow().fmt(f)
    }
}

impl Display for BatteryData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "battery at {}% charge", 100.0 * self.charge_level / self.capacity)
    }
}

fn max(x: f32,y: f32)->f32 {
    if x > y { x } else { y }
}

fn min(x: f32,y: f32)->f32 {
    if x < y { x } else { y }
}

impl Component for Battery {
    fn get_potential_supply(&self, time: &GameTime) -> Resources {
        let data = self.data.borrow();
        Resources::electric(min(data.charge_level,data.discharge_rate))
    }
    fn get_potential_consumption(&self, resources: &Resources, time: &GameTime) -> Resources {
        let data = self.data.borrow();
        let mut resources = Resources::default();
        if !self.data.borrow().is_charged() {
            resources.electricity -= min(resources.electricity,min(data.charge_rate,data.capacity - data.charge_level));
        }
        resources
    }
    fn supply_on_demand(&mut self, resources: &Resources, _: &GameTime) -> Resources {
        let mut data = self.data.borrow_mut();
        let mut output = Resources::default();
        if resources.electricity < 0.0 {
               let charge_amt = min(min(data.discharge_rate,-resources.electricity),data.charge_level);
               output.electricity += charge_amt;
               data.charge_level -= charge_amt;
       }
       output
    }
    fn consume_on_demand(&mut self, resources: &Resources, _: &GameTime) -> Resources {
        let mut data = self.data.borrow_mut();
        let mut output = Resources::default();
        if resources.electricity > 0.0 {
                let charge_amt = min(min(data.charge_rate,resources.electricity),data.capacity - data.charge_level);
                output.electricity -= charge_amt;
                data.charge_level += charge_amt;
        }
        output
    }
}

pub struct Radiator {
    ambient: f32,
}

impl Radiator {
    fn new_component() -> Rc<RefCell<Box<Component>>> {
        Rc::new(RefCell::new(Box::new(Radiator{ambient: 0.0}) as Box<Component>))
    }
}

impl Component for Radiator {
    fn get_fixed_processing(&mut self, _: &GameTime) -> Resources {
        self.ambient *= 0.9;
        Resources::default()
    }
    fn get_potential_supply(&self, time: &GameTime) -> Resources {
        Resources {
            electricity: 0.0, heat: 9001.0, deuterium: 0.0,
        }
    }
    fn consume_on_demand(&mut self,resources: &Resources, time: &GameTime) -> Resources {
        let mut output = Resources::default();
        let ratio = resources.heat / self.ambient;
        let amt = ratio - 1.0;
        if amt > 0.0 {
            let clamped = min(amt, 0.5); //dump at most 50% of heat
            let remove = clamped * (resources.heat - self.ambient);
            output.heat -= remove;
            self.ambient += remove * 0.1; //aproximate local heating
        }
        output
    }
}

impl Display for Radiator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ambient temp: {}K", self.ambient)
    }
}

pub struct DeuteriumTank {
    storage: f32,
}

impl Component for DeuteriumTank {
    fn get_potential_supply(&self, time: &GameTime) -> Resources {
        Resources {
            electricity: 0.0, heat: 0.0, deuterium: self.storage,
        }
    }
    fn supply_on_demand(&mut self, resources: &Resources, time: &GameTime) -> Resources {
        let mut output = Resources::default();
        if resources.deuterium < 0.0 {
            let delta = min(-resources.deuterium, self.storage);
            self.storage -= delta;
            output.deuterium += delta;
        }
        output
    }
}

impl Display for DeuteriumTank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "available deuterium: {}g", self.storage)
    }
}

pub struct Laser {
    fired: bool,
}

impl Laser {
    fn new_component() -> Rc<RefCell<Box<Component>>> {
        Rc::new(RefCell::new(Box::new(Laser{fired: false}) as Box<Component>))
    }
}

impl Display for Laser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.fired {
            write!(f, "laser fired!")
        } else {
            write!(f, "laser didn't fire.")
        }
    }
}

impl Component for Laser {
    fn get_potential_consumption(&self, resources: &Resources, time: &GameTime) -> Resources {
        let mut output = Resources::default();
        if resources.electricity >= 500.0 {
            output.electricity -= 500.0;
        }
        output
    }
    fn consume_on_demand(&mut self,resources: &Resources, time: &GameTime) -> Resources {
        let mut output = Resources::default();
        if resources.electricity >= 500.0 {
            self.fired = true;
            output.electricity -= 500.0;
            output.heat += 5.0; //laser is 99% efficient
        } else {
            self.fired = false;
        }
        output
    }
}

pub struct ComponentManager {
    components: Vec<Rc<RefCell<Box<Component>>>>,
    supply: Vec<usize>,
    demand: Vec<usize>,
}

impl ComponentManager {
    fn update(&mut self, resources: &mut Resources){
        assert!(self.components.len() == self.supply.len());
        assert!(self.components.len() == self.demand.len());
        let time = GameTime {ms: 1};
        for c in &self.components {
            *resources += c.borrow_mut().get_fixed_processing(&time);
        }
        let mut potential_supply = resources.clone();
        for c in &self.components {
            potential_supply += c.borrow_mut().get_potential_supply(&time);
        }
        let mut potential_consumption = resources.clone();
        for c in &self.components {
            let output = c.borrow_mut().get_potential_consumption(&potential_supply,&time);
            potential_supply += output;
            potential_consumption += output;
        }
        //println!("fixed state: {}",resources);
        //println!("potential demand: {}",potential);
        for c in &self.supply {
            let output = self.components[*c].borrow_mut().supply_on_demand(&potential_consumption, &time);
            //println!("{} supplied {}",*self.components[*c].borrow(),output);
            *resources += output;
            potential_consumption += output;
        }
        //println!("after supply: {}",resources);
        for c in &self.demand {
            *resources += self.components[*c].borrow_mut().consume_on_demand(resources, &time);
        }
        //println!("after demand: {}",resources);
    }
}

impl Display for ComponentManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f,"{{ "));
        for c in &self.components {
            try!(c.borrow().fmt(f));
            try!(write!(f," : "));
        }
        try!(write!(f," }}"));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::boxed::Box;
    use std::thread::sleep;
    use std::time::Duration;
    #[test]
    fn it_works() {
        let mut mgr = ComponentManager{ components: Vec::new(),
            supply: vec![0,1,2,3,4,5],
            demand: vec![5,0,2,1,3,4],
        };
        mgr.components.push(FusionReactor::new_component(100.0));
        mgr.components.push(Capacitor::new_component(500.0));
        let battery = Box::new(Battery{data:BatteryData::new(200.0, 0.9)});
        let bat_dat = battery.data.clone();
        mgr.components.push(Rc::new(RefCell::new(battery as Box<Component>)));
        //mgr.update();
        //println!("{}\t{}",bat_dat.borrow().is_charged(),*bat_dat.borrow());*/
        //let battery = Rc::new(RefCell::new(Box::new(Battery::new(100000.0))));
        //let bat_trait: Rc<RefCell<Box<Component>>> = battery;
        //mgr.components.push(bat_trait);
        mgr.components.push(Rc::new(RefCell::new(Box::new(DeuteriumTank{storage: 100.0}))));
        mgr.components.push(Radiator::new_component());
        print!("{}[2J", 27 as char); //clear the screen
        let mut resources = Resources::default();
        println!("initial state: {}",mgr);
        /*while !bat_dat.borrow().is_charged() {
            mgr.update(&mut resources);
            print!("{} -> System heat is {}J\n",mgr,resources.heat);
            //sleep(Duration::new(1,0));
        }
        println!("Charged!");*/
        mgr.components.push(Laser::new_component());
        //mgr.supply.push(5);
        //mgr.demand.insert(0,5);
        for i in 1..40 {
            mgr.update(&mut resources);
            print!("{} {}\n",mgr,resources);
        }
        println!("Done! End state is {}", resources);
    }
}
