//todo: do not consider future turn when it's the last turn of the simulation
//todo: add more comment to the complicated functions
//todo: add a feature that tells you what turn do you run out of cards in averge 

extern crate colored;
extern crate fastrand;

use std::sync::{Mutex, Arc};
use std::thread;
use std::error::Error;
use std::collections::HashSet;
use std::io::Read;
use std::fs::File;
use colored::*;
use std::fmt;
use std::cmp::Ordering;


#[derive(Clone)]
#[derive(Debug, PartialEq, Eq, Hash)]
enum CardPower
{
    Strong,
    Normal,
    Weak,
}

impl Ord for CardPower
{
    fn cmp(&self, other:&Self) -> Ordering
    {
        let a = match self
        {
            CardPower::Strong=>3,
            CardPower::Normal=>2,
            CardPower::Weak=>1,
        };

        let b = match other
        {
            CardPower::Strong=>3,
            CardPower::Normal=>2,
            CardPower::Weak=>1,
        };

        a.cmp(&b)
    }
}

impl PartialOrd for CardPower
{
    fn partial_cmp(&self, other:&Self) -> Option<Ordering>
    {
        Some(self.cmp(&other))
    }
}

impl CardPower
{
    fn to_char(&self) -> char
    {
        match self
        {
            CardPower::Strong=>'s',
            CardPower::Normal=>'n',
            CardPower::Weak=>'w',
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum CardLocation
{
    InDeck,
    InHand,
    NoWhere,
}

#[derive(Debug, PartialEq, Eq)]
enum CommandResult
{
    Ok,
    Err(String),
    End,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Hero
{
    Warrior,
    Priest,
    Hunter,
    Warlock,
    Mage,
    Rogue,
    Shaman,
    Paladin,
    DemonHunter,
    Druid,
}

impl Hero
{
    fn hero_power_value(&self) -> f64
    {
        match self
        {
            Hero::Warrior => 0.0,
            Hero::Priest => 0.0,
            Hero::Hunter => 0.0,
            Hero::Warlock => 0.0,
            Hero::Mage => 0.5,
            Hero::Rogue => 1.2,
            Hero::Shaman => 0.8,
            Hero::Paladin => 1.0,
            Hero::DemonHunter => 0.5,  //per mana
            Hero::Druid => 0.5,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Card
{
    mana:i8,
    card_power:CardPower
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self.mana.to_string(); 

        let result = match self.card_power
        {
            CardPower::Strong => result.green(),
            CardPower::Normal => result.yellow(),
            CardPower::Weak => result.red(),
        };

        let result = match self.mana 
        {
            -1=>format!("{}", "coin".yellow()),
            _=>format!("{}", result),
        };
        f.pad(&result)
     }
}


impl Card
{
    ///create a card with string.  
    /// 
    ///s for strong, n for normal, w for weak  
    /// 
    ///example create("n2"); return 2mana normal card  
    /// 
    ///if the letter is not given , assume it's normal
    fn create(card_str:&str) -> Option<Card>
    {
        let power;
        let mana;
        match card_str.chars().nth(0)
        {
            Some('s')=>power=CardPower::Strong,
            Some('n')=>power=CardPower::Normal,
            Some('w')=>power=CardPower::Weak,
            Some(num)=>
            {
                if num >= '0' && num <= '9'
                {
                    power=CardPower::Normal;
                }
                else
                {
                    return None;
                }
            },
            None=>return None,
        };
        match card_str.chars().filter(|&x| x>='0' && x<='9').collect::<String>().parse()
        {
            Ok(n)=>mana=n,
            Err(_)=>return None,
        }
        let card = Card{mana:mana, card_power:power};
        Some(card)
    }

    ///create cards with same stats
    /// 
    ///example: 3s4 means create 3 strong 4mana-cost cards
    fn create_cards(cards_str:&str) -> Option<Vec<Card>>
    {
        let pos = match cards_str.chars().position(|x| x=='n' || x=='s' || x=='w')
        {
            Some(p)=>p,
            None=>return None,
        };
        let num = match cards_str[..pos].parse()
        {
            Ok(n)=>n,
            Err(_)=>return None,
        };
        let mut result = Vec::new();
        for _ in 0..num
        {
            let card = match Card::create(&cards_str[pos..])
            {
                Some(c)=>c,
                None=>return None,
            };
            result.push(card);
        }
        Some(result)
    }

}

impl Ord for Card
{
    fn cmp(&self, other:&Self) -> Ordering
    {
        match self.mana.cmp(&other.mana)
        {
            Ordering::Equal=>
            {
                self.card_power.cmp(&other.card_power) 
            },
            other_order => other_order,
        }
    }
}

impl PartialOrd for Card
{
    fn partial_cmp(&self, other:&Self) -> Option<Ordering>
    {
        Some(self.cmp(other))
    }
}



#[derive(Clone)]
struct Dealer
{
    cards:Vec<Card>,
    card_location:Vec<CardLocation>, 
}

impl fmt::Debug for Dealer
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        let mut result = String::new();
        for card in self.cards.iter()
        {
            result += format!("mana:{} , power:{:?}\n", card.mana, card.card_power).as_str();
        }
        f.pad(&result)
    }
}

impl Dealer
{
    ///save the deck
    fn save(&self, filename:String)
    {
        let save_data = self.cards.iter().map(|x| format!("{}{}",x.card_power.to_char(), x.mana)).collect::<Vec<String>>().join(" ");
        std::fs::write(filename, save_data).expect("failed to write file");
    }

    ///load deck from file
    fn load(&mut self, filename:String) -> CommandResult
    {
        let mut file = match File::open(filename)
        {
            Ok(f)=>f,
            Err(_)=>return CommandResult::Err("failed to read file".to_string()),
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents)
        {
            Ok(_)=>(),
            Err(_)=>return CommandResult::Err("failed to read file".to_string()),
        }
        self.clear();
        for card in contents.trim().split(' ')
        {
            match Card::create(card)
            {
                Some(c)=>self.insert_card(c),
                None=>return CommandResult::Err("failed to create card".to_string()),
            };
        }
        CommandResult::Ok
    }

    fn insert_card(&mut self, card:Card)
    {
        self.cards.push(card);
        self.card_location.push(CardLocation::InDeck);
    }

    ///fill the deck with high cost cards
    fn fill_deck(&mut self) -> CommandResult
    {
        let len = self.cards.len();
        if len > 30
        {
            return CommandResult::Err("too much cards!".to_string());
        }
        self.insert_cards(format!("{}n9", 30-len).as_str());
        CommandResult::Ok
    }

    ///clear the entire deck(including the vector)
    fn clear(&mut self)
    {
        self.cards.clear();
        self.card_location = Vec::new();
    }

    ///reset card location back to the deck. delete the coin if there is one.
    fn reset_deck(&mut self)
    {
        self.card_location.fill(CardLocation::InDeck);
        self.cards = self.cards.clone().into_iter().filter(|x| x.mana!=-1).collect();
        let num_need_pop =  self.card_location.len() - self.cards.len();
        for _ in 0..num_need_pop
        {
            self.card_location.pop();
        }
    }

    ///insert cards
    /// 
    /// # example
    /// 
    /// ```
    /// //insert 4 normal 2 drops, 5 normal 3 drops, 1 weak 1 drop, 2 strong 4 drops
    /// self.insert_cards("4n2 5n3 1w1 2s4"); 
    /// ```
    fn insert_cards(&mut self, cards_str:&str) -> CommandResult
    {
        let cards = match Card::create_cards(cards_str)
        {
            Some(c)=>c,
            None=>return CommandResult::Err("failed to create card".to_string()),
        };
        for card in cards.into_iter()
        {
            self.insert_card(card);
        }
        CommandResult::Ok
    }

    ///draw a card randomly. return the position of the card drew.
    fn draw_card(&mut self, card_locations:Option<&mut Vec<CardLocation>>) -> usize
    {
        let locations = match card_locations
        {
            Some(x)=>x,
            None=>&mut self.card_location,
        };
        let len = locations.iter().filter(|x| x == &&CardLocation::InDeck).count();
        //eprintln!("the len is {}", len);
        let pos = fastrand::usize(..len);
        let (pos_in_deck,location) = locations.iter_mut().enumerate().filter(|(_,lo)| lo == &&CardLocation::InDeck).nth(pos).unwrap();
        *location = CardLocation::InHand;
        pos_in_deck
    }

    fn new() -> Dealer
    {
        Dealer{cards:Vec::new(), card_location:Vec::new()}
    }

    ///change a card from deck to hand
    fn deck_to_hand(&mut self, card_pos:usize) -> bool
    {
        match self.card_location.get_mut(card_pos)
        {
            Some(lo)=>*lo=CardLocation::InHand,
            None=>return false,
        }
        return true;
    }

    ///get the position of a card
    ///location:the location of the searching card.(example: CardLocation::InDeck)
    ///card_locations:locations of all cards. if it's None, use self.card_location
    fn get_card_pos(&self, card:Card, location:&CardLocation, card_locations:Option<&Vec<CardLocation>>) -> Option<usize>
    {
        let card_locations = match card_locations
        {
            Some(x)=>x.to_vec(),
            None=>self.card_location.clone(),
        };
        self.cards.iter().zip(card_locations.iter())
                .position(|(c,lo)| c.mana == card.mana && c.card_power == card.card_power && lo == location)
    }

    ///change 「position in deck」 vector to 「struct Card」 vector
    fn position_to_cards(&self, positions:&Vec<usize>) -> Vec<Card>
    {
        positions.into_iter().map(|&p| self.cards[p].clone()).collect()
    }


    fn get_hand(&self, card_locations:&Vec<CardLocation>) -> Vec<Card>
    {
        self.cards.clone().into_iter().zip(card_locations.iter())
                .filter(|(_,lo)| lo == &&CardLocation::InHand).map(|(c,_)| c).collect()
    }

    ///change 「struct Card」 vector to 「position in deck」 vector
    ///can't work with CardLocation::NoWhere
    fn cards_to_position(&mut self, cards:Vec<Card>, locations:CardLocation, card_locations:Option<&Vec<CardLocation>>) -> Option<Vec<usize>>
    {
        let mut pos_set = Vec::new();
        let locations_temp = self.card_location.clone();
        for card in cards.into_iter()
        {
            let pos = match self.get_card_pos(card.clone(), &locations, card_locations)
            {
                Some(p)=>p,
                None=>return None,
            };
            pos_set.push(pos);
            self.card_location[pos] = CardLocation::NoWhere;
        }
        self.card_location = locations_temp;
        Some(pos_set)
    }


    ///delete or add coin depending on play_order
    fn adjust_coin(&mut self, play_order:PlayOrder)
    {
        let have_coin = self.cards.iter().any(|x|x.mana==-1);
        match (play_order,have_coin)
        {
            (PlayOrder::First,true)=>
            {
                let coin_pos = self.cards.iter().position(|x| x.mana==-1).unwrap();
                self.cards.swap_remove(coin_pos);
                self.card_location.swap_remove(coin_pos);
            },
            (PlayOrder::Second,false)=>
            {
                self.cards.push(Card{mana:-1, card_power:CardPower::Normal});
                self.card_location.push(CardLocation::InDeck);
            },
            _=>(),
        }
    }

    fn sort_deck(&mut self)
    {
        // self.cards.iter().zip(self.card_location)
        self.cards.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
enum PlayOrder
{
    First,
    Second,
}

impl PlayOrder
{
    fn get_start_hand_size(&self) -> u8
    {
        match self
        {
            PlayOrder::First=>3,
            PlayOrder::Second=>4,
        }
    }

    ///for iterate all patterns use
    ///works like 000, 001, ... , 111
    fn get_pattern_int(&self) -> u8
    {
        match self
        {
            PlayOrder::First=>8,
            PlayOrder::Second=>16,
        }
    }

    fn flip_the_coin() -> PlayOrder
    {
        match fastrand::bool()
        {
            true=>PlayOrder::First,
            false=>PlayOrder::Second,
        }
    }
}

#[derive(Clone)]
struct Simulator
{
    cycle_reps:i32,
    dealer:Dealer,
    hand:Vec<usize>,
    score:f64,
    maxturn:u8,
    max_search_depth:u8,
    hero:Hero,
    play_order:PlayOrder,
    play_card_bonus:i8,
}

impl Simulator
{
    ///clear everything including the deck
    fn clear(&mut self)
    {
        self.score=0.0;
        self.play_order = PlayOrder::First;
        self.hand=Vec::new();
        self.dealer.clear();
    }

    ///run simulation of a giving hand for cycle_reps times. return averge score.
    fn start_simulation(&mut self, kept_hand:Vec<usize>, do_print:bool) -> Option<f64>
    {    
        let result_score = Arc::new(Mutex::new(0.0));
        let mut handles = vec![];
        for _ in 0..4
        {
            let mut sim = self.clone();
            let result_score = Arc::clone(&result_score);
            let kept_hand = kept_hand.clone();
            let handle = thread::spawn(move || 
            {
                let mut score_this_thread = 0.0;
                for _ in 0..sim.cycle_reps/4
                {
                    sim.score=0.0;
                    sim.reset();

                    let _result = sim.set_start_hand(&kept_hand);
                    // if result == false
                    // {
                    //     return None;
                    // }

                    if do_print {eprintln!("\nthe kept_hand is {:?}", sim.dealer.get_hand(&sim.dealer.card_location));}


                    for turn in 1..=sim.maxturn
                    {
                        sim.draw_card();
                        let card_drew = sim.hand.last();
                        if do_print {
                            println!("in turn {} the draw is {:?} the hand is {:?}", turn, card_drew, sim.dealer.get_hand(&sim.dealer.card_location));
                        }

                        let score_a_turn = sim.play_a_turn(None, turn as i8, sim.max_search_depth, do_print);
                        sim.score += score_a_turn;
                    }
                    score_this_thread += sim.score;
                    if do_print {eprintln!("the score this rep is {}", sim.score);}
                }
                let mut score = result_score.lock().unwrap();
                *score += score_this_thread;
            });
            handles.push(handle);
        }

        for handle in handles
        {
            handle.join().unwrap();
        }

        let result = *result_score.lock().unwrap();
        Some(result/self.cycle_reps as f64)
    }


    fn draw_card(&mut self)
    {
        self.hand.push(self.dealer.draw_card(None));
    }

    fn new(cycle_reps:i32, maxturn:u8, max_search_depth:u8, play_card_bonus:i8) -> Simulator
    {
        Simulator{
                    cycle_reps,
                    dealer:Dealer::new(),
                    hand:Vec::new(),
                    score:10.0,
                    maxturn, 
                    hero:Hero::Warrior,
                    max_search_depth,
                    play_order:PlayOrder::First,
                    play_card_bonus,
                }
    }

    fn set_start_hand(&mut self,kept_hand:&Vec<usize>) -> bool
    {
        //reset deck. if there is coin in the deck, delete it.
        self.dealer.reset_deck();
        match self.dealer.cards.iter().position(|x| x.mana == -1)
        {
            Some(p)=>{self.dealer.cards.remove(p);self.dealer.card_location.remove(p);},
            None=>(),
        };

        //draw the kept hand from deck.
        for &card_pos in kept_hand.iter()
        {
            let result = self.dealer.deck_to_hand(card_pos);
            if result == false
            {
                return false;
            }
        }

        //set kept_hand
        self.hand = kept_hand.to_vec();

        //draw the rest
        for _ in 0..self.play_order.get_start_hand_size() as usize - kept_hand.len()
        {
            self.draw_card();
        }

        //add coin according to play order
        if self.play_order == PlayOrder::Second
        {
            self.dealer.insert_card(Card{mana:-1, card_power:CardPower::Normal});
            *self.dealer.card_location.last_mut().unwrap() = CardLocation::InHand;
            self.hand.push(self.dealer.cards.len()-1);
        }
        true
    }

    ///get all play patterns that should be tried.  
    /// 
    ///not including those which have more mana-cost in total than provide
    /// 
    ///no duplicate plays
    /// 
    ///if a play is returned , the subset of play should not be returned. example:if [2,3] is valid, then don't try play [], [2], [3]
    fn get_all_play_patterns(&self, hand:Vec<Card>, mana_max:i8, mana_min:i8) -> Vec<Vec<Card>>
    {
        //take 0-mana cards out since they are always gonna be played
        let no_zero_hand = hand.clone().into_iter().filter(|x| x.mana != 0).collect::<Vec<Card>>();
        let zero_mana_cards = hand.clone().into_iter().filter(|x| x.mana == 0).collect::<Vec<Card>>();
        
        let mut patterns_for_now = self.get_all_plays(no_zero_hand, mana_max, mana_min);
        patterns_for_now = Simulator::remove_duplicate_plays(patterns_for_now);
        Simulator::add_play_nothing(&mut patterns_for_now);

        //put those 0-mana cards back
        for pattern in patterns_for_now.iter_mut()
        {
            pattern.append(&mut zero_mana_cards.clone());
        }
        patterns_for_now
    }

    ///a step for function get_all_play_patterns
    fn get_all_plays(&self, hand:Vec<Card>, mana_max:i8, mana_min:i8) -> Vec<Vec<Card>>
    {
        let mana_max = match mana_max < 0
        {
            true => 0,
            false => mana_max
        };

        let mana_min = match mana_min < 0
        {
            true => 0,
            false => mana_min
        };

        let mut hand = hand.clone();
        if hand.is_empty()
        {
            return Vec::new();
        }

        let have_coin = match hand.iter().position(|x| x.mana==-1)
        {
            Some(p)=>{hand.swap_remove(p);true},
            None=>false,
        };


        hand = hand.into_iter().filter(|x| x.mana<=mana_max+1).collect();
        //hand.sort_by(|a,b| a.mana.cmp(&b.mana));

        let mut patterns_for_now = Vec::new();
        if have_coin
        {
            patterns_for_now = self.get_all_plays(hand.clone(), mana_max+1, mana_max+1);
            for pattern in patterns_for_now.iter_mut()
            {
                pattern.push(Card{mana:-1, card_power:CardPower::Normal});
            }
            //eprintln!("the patterns_with_coin is {:?}", patterns_for_now);
        }

        //take cards that not bigger than mana_max
        hand = hand.into_iter().filter(|x| x.mana<=mana_max).collect();
        if hand.is_empty()
        {
            return patterns_for_now;
        }

        //eprintln!("the hand is {:?}", hand.clone());
        let last = hand.pop().unwrap();
        if hand.is_empty()
        {
            if last.mana>=mana_min
            {
                patterns_for_now.append(&mut vec![vec![last.clone()]]);
            }
            //eprintln!("patterns with coin before return is {:?}", patterns_for_now.clone());
            return patterns_for_now;
        }

        let mut min = mana_min - last.mana;

        //get all patterns that include the last card
        // eprintln!("last={:?}, hand={:?}, max={}, min={}", last, hand.clone(), mana_max-last.mana, min);
        let mut patterns1 = self.get_all_plays(hand.clone(), mana_max-last.mana, min);
        //eprintln!("patterns1={:?}", patterns1);
        for pattern in patterns1.iter_mut()
        {
            pattern.push(last.clone());
        }
        if patterns1.is_empty() && last.mana>=mana_min
        {
            patterns1.push(vec![last.clone()]);
        }

        //get all patterns that not include the last card and is not a subset of previous set above
        min = match (mana_min > mana_max - last.mana, mana_max - last.mana == 0)
        {
            (true,_) => mana_min,
            (false,true) => 0,
            (false,false) => 
            {
                if last.mana == hand.last().unwrap().mana && last.card_power != hand.last().unwrap().card_power
                {
                    mana_max - last.mana + 1
                }
                else
                {
                    mana_max - last.mana
                }
            }
        };
        // eprintln!("last={:?}, hand={:?}, max={}, min={}", last, hand.clone(), mana_max, min);
        let mut patterns2 = self.get_all_plays(hand.clone(), mana_max, min);
        patterns_for_now.append(&mut patterns1);
        patterns_for_now.append(&mut patterns2);

        patterns_for_now
    }

    ///a step for function "get_all_play_patterns"
    /// 
    ///remove_duplicate_plays
    fn remove_duplicate_plays(plays:Vec<Vec<Card>>) -> Vec<Vec<Card>>
    {
        let mut map = HashSet::new();
        for play in plays.into_iter()
        {
            map.insert(play);
        }
        map.into_iter().collect::<Vec<Vec<Card>>>()
    }

    ///a step for function "get_all_play_patterns"
    /// 
    ///add 「play nothing」 if (「playlist is empty」 or 「every play contains a coin」)
    fn add_play_nothing(plays:&mut Vec<Vec<Card>>)
    {
        if !plays.iter().any(|x| x.iter().all(|x| x.mana >= 0))
        {
            plays.push(vec![]);
        }
    }


    ///play a turn
    /// 
    /// try every reasonable play according to hand(get from card_locations) and mana
    /// 
    /// look forward for depth turns
    /// 
    /// do the play with highest score
    fn play_a_turn(&mut self, card_locations:Option<&Vec<CardLocation>>, mana:i8, depth:u8, do_print:bool) -> f64
    {
        let do_orignal = match card_locations
        {
            Some(_)=>false,
            None=>true,
        };

        let card_locations = match card_locations
        {
            Some(x)=>x.to_vec(),
            None=>self.dealer.card_location.clone(),
        };

        if do_print && depth == self.max_search_depth
        {
            let mut card_drew_pos = 0;
            for card_pos in self.hand.iter().rev()
            {
                if self.dealer.cards[*card_pos].mana != -1
                {
                    card_drew_pos = *card_pos;
                    break;
                }
            }
            println!("\nin turn {},the draw is [{:?}] ,the hand is :{:?}", mana, self.dealer.cards[card_drew_pos], self.dealer.get_hand(&card_locations));
        }

        let hand:Vec<Card> = self.dealer.cards.clone().into_iter().zip(card_locations.iter())
                                    .filter(|(_,lo)| lo == &&CardLocation::InHand).map(|(c,_)|c)
                                    .collect();

        //get all reasonable plays
        let all_plays = self.get_all_play_patterns(hand.clone(), mana, 0);

        //println!("in turn {}, the hand is {:?}, all patterns are {:?}", mana, hand, all_plays);
        if do_print && depth == self.max_search_depth
        {
            println!("all patterns are {:?}", all_plays);
        }
        let mut max_score = -10.0;
        let mut best_play = Vec::new();


        //try every play 
        for play in all_plays.into_iter()
        {
            if do_print
            {
                eprintln!("trying play {:?}", play);
            }

            let mut score = 10.0;


            let mana_waste:i8 = mana as i8 - play.iter().map(|x| x.mana).sum::<i8>();
            score -= mana_waste as f64;


            if (self.hero == Hero::DemonHunter && mana_waste == 1 && mana != 1) || (self.hero != Hero::DemonHunter && mana_waste == 2)
            {
                if do_print {eprintln!("use hero power");}
                score += self.hero.hero_power_value();
            }


            //play those cards
            let mut result_card_location = card_locations.clone();
            for card in play.iter()
            {
                //eprintln!("playing card {:?}, hand is {:?}", card, self.dealer.get_hand(&result_card_location));
                let card_pos = self.dealer.get_card_pos(card.clone(), &CardLocation::InHand, Some(&result_card_location)).unwrap();
                score += self.play_a_card(card_pos, Some(&mut result_card_location));
            }



            
            let mut score_sum = 0.0;

            //try 10 draws, take average
            if depth > 1  
            {
                for _ in 0..10
                {
                    let mut locations_temp = result_card_location.clone();
                    self.dealer.draw_card(Some(&mut locations_temp));
                    let print_next_turn =false;
                    let future_turn_score = self.play_a_turn(Some(&locations_temp), mana+1, depth-1, print_next_turn);
                    score_sum += future_turn_score;
                }
            }

            let score_this = score;
            let score_future = score_sum/10.0;
            score = score_this + score_future;


            if do_print {println!("the score of this try is {:.3}. (this turn:{:.3} + future:{:.3})", score, score_this, score_future);}

            if score > max_score
            {
                best_play = play;
                max_score = score;
            }
        }

        if do_print
        {
            println!("the best play of hand {:?} in turn {} is: {:?}", 
                            self.dealer.get_hand(&card_locations), 
                            mana, 
                            best_play
                    );
        }


        //do the best play
        if do_orignal == true
        {
            max_score = 10.0;

            let mana_waste:i8 = mana as i8 - best_play.iter().map(|x| x.mana).sum::<i8>();
            max_score -= mana_waste as f64;


            if (self.hero == Hero::DemonHunter && mana_waste == 1 && mana != 1) || (self.hero != Hero::DemonHunter && mana_waste == 2)
            {
                if do_print {println!("use hero power");}
                max_score += self.hero.hero_power_value();
            }

            for card in best_play.iter()
            {
                let card_pos = self.dealer.get_card_pos(card.clone(), &CardLocation::InHand, None).unwrap();
                max_score += self.play_a_card(card_pos, None);
            }
        }
        max_score
    }

    ///show how to play a hand without changing any data in Simulator
    fn play_a_hand(&mut self, hand:Vec<Card>, mana:i8, do_print:bool)
    {
        let mut locations = vec![CardLocation::InDeck].repeat(self.dealer.card_location.len());
        for card in hand.iter()
        {
            if card.mana == -1
            {
                locations.push(CardLocation::InHand);
            }
            let pos = self.dealer.cards.iter().zip(locations.iter()).position(|(c,l)| l==&CardLocation::InDeck && c == card).unwrap();
            locations[pos] = CardLocation::InHand;
        }
        self.play_a_turn(Some(&locations), mana, self.max_search_depth, do_print);
    }


    ///if card_location is None, use the orignal one (self.dealer.card_location). change self.hand only if it's the orignal.
    fn play_a_card(&mut self, pos_in_deck:usize, card_locations:Option<&mut Vec<CardLocation>>) -> f64
    {
        //if it's the orignal one ,change the hand in simulator also.
        if card_locations == None
        {
            self.hand.swap_remove(self.hand.iter().position(|x| x == &pos_in_deck).unwrap());
        }

        let locations = match card_locations
        {
            Some(x)=>x,
            None=>&mut self.dealer.card_location,
        };
        locations[pos_in_deck] = CardLocation::NoWhere;

        match self.dealer.cards[pos_in_deck].card_power
        {
            CardPower::Strong=>0.5 + self.play_card_bonus as f64,
            CardPower::Normal=>0.0 + self.play_card_bonus as f64,
            CardPower::Weak=>-0.5 + self.play_card_bonus as f64,
        }
    }


    ///reset the hand and deck. mostly to start a new game by the same deck and setting
    fn reset(&mut self)
    {
        self.dealer.reset_deck();
        self.hand = Vec::new();
    }

    fn set_hero(&mut self, word: &str) -> CommandResult
    {
        match word 
        {
            "wr" | "warrior"=>self.hero=Hero::Warrior,
            "wl" | "warlock"=>self.hero=Hero::Warlock,
            "pr" | "priest"=>self.hero=Hero::Priest,
            "dr" | "druid"=>self.hero=Hero::Druid,
            "ma" | "mage"=>self.hero=Hero::Mage,
            "pa" | "paladin"=>self.hero=Hero::Paladin,
            "sh" | "shaman"=>self.hero=Hero::Shaman,
            "ro" | "rogue"=>self.hero=Hero::Rogue,
            "hu" | "hunter"=>self.hero=Hero::Hunter,
            "dh" => self.hero=Hero::DemonHunter,
            _=>return CommandResult::Err("it's not a hero".to_string()),
        }
        println!("the hero is set to {:?}", self.hero);
        CommandResult::Ok
    }


    ///input a hand. print all possible mulligan score
    /// 
    ///return the best move. 
    /// 
    ///example:return '101' means keep the first and third card
    fn solve_mull(&mut self, hand:Vec<usize>) -> String
    {
        // let hand_size = self.play_order.get_start_hand_size();

        //if hand-length is lower than hand size , fill the hand with high-cost card
        // hand.append(&mut Card::create_cards(format!("{}n10", hand_size as usize - hand.len()).as_str()).unwrap());
        
        let mut hand = hand.clone();
        hand.sort();

        //try all patterns of mulligan
        let mut result = Vec::new();
        let mut hand_set = std::collections::HashSet::new();
        for index in 0..self.play_order.get_pattern_int()
        {
            let mut result_hand = Vec::new();
            let pattern = match self.play_order
            {
                PlayOrder::First=>format!("{:03b}", index).chars().collect::<String>(),
                PlayOrder::Second=>format!("{:04b}", index).chars().collect::<String>(),
            };
            for (i,c) in pattern.chars().enumerate()
            {
                if c == '1'
                {
                    result_hand.push(hand[i].clone());
                }
            }
            let hand_by_cards = self.dealer.position_to_cards(&result_hand);
            let already_tested = !hand_set.insert(hand_by_cards.clone());
            let pass_string = match already_tested
            {
                true => "(pass)",
                false => "",
            };
            println!("{}/{}{}", index+1, self.play_order.get_pattern_int(), pass_string);

            if !already_tested
            {
                let do_print = false;
                let score = self.start_simulation(result_hand.clone(), do_print).unwrap();
                // println!("the score of {:?} is :{:.3}", self.dealer.position_to_cards(&result_hand), score);
                result.push((pattern,score,hand_by_cards));
            }
        }
        result.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

        let total_score = 10.0 * self.maxturn as f64;
        println!("the total base score is {}, {} for every mana waste. +{} for every card played", total_score.to_string().yellow(), "-1".to_string().red(), self.play_card_bonus.to_string().green());
        for line in result.iter()
        {
            println!("the score of {:?} is :{}", line.2, format!("{:.3}", line.1).yellow());
        }
        let the_best = result.last().unwrap();
        // let the_best = result.into_iter().max_by(|x,y|x.1.partial_cmp(&y.1).unwrap()).unwrap();

        println!("the best move is:");
        for (&p,k) in hand.iter().zip(the_best.0.chars())
        {
            print!("card:{:?}  ", self.dealer.cards[p]);
            if k=='0'
            {
                println!("{}", "not-keep".red());
            }
            else
            {
                println!("{}", "keep".green());
            }
        }
        self.dealer.reset_deck();
        println!("the score is {:.3}", the_best.1);
        the_best.0.clone()
    }

    ///set play order. then add or delete coin according to play order
    fn set_play_order(&mut self, order:PlayOrder)
    {
        self.play_order = order.clone();
        self.dealer.adjust_coin(order);
    }

    fn sim_common_pattern(&mut self)
    {
        let kept_hand = vec![
                                (vec![Card{mana:1, card_power:CardPower::Normal}],Vec::new(),"keeping 1"), //单留1费
                                (vec![Card{mana:1, card_power:CardPower::Strong}],Vec::new(),"keeping 1(strong)"), //单留1费(强)
                                (vec![Card{mana:2, card_power:CardPower::Normal}],Vec::new(),"keeping 2"), //单留2费
                                (vec![Card{mana:3, card_power:CardPower::Normal}],Vec::new(),"keeping 3"), //单留3费
                                (vec![Card{mana:3, card_power:CardPower::Strong}],Vec::new(),"keeping 3(strong)"), //单留3费(强)
                                (vec![Card{mana:4, card_power:CardPower::Normal}],Vec::new(),"keeping 4"), //单留4费
                                (vec![Card{mana:4, card_power:CardPower::Strong}],Vec::new(),"keeping 4(strong)"), //单留4费(强)
                                (vec![Card{mana:2, card_power:CardPower::Normal},Card{mana:2, card_power:CardPower::Normal},Card{mana:3, card_power:CardPower::Normal}]
                                    ,vec![Card{mana:2, card_power:CardPower::Normal},Card{mana:3, card_power:CardPower::Normal}],"having 2 3, keeping 2"), //有23,留2
                                (vec![Card{mana:2, card_power:CardPower::Normal},Card{mana:3, card_power:CardPower::Normal},Card{mana:3, card_power:CardPower::Normal}]
                                    ,vec![Card{mana:2, card_power:CardPower::Normal},Card{mana:3, card_power:CardPower::Normal}],"having 2 3, keeping 3"), //有23,留3
                                (vec![Card{mana:1, card_power:CardPower::Normal}, Card{mana:3, card_power:CardPower::Normal}],vec![Card{mana:1, card_power:CardPower::Normal}],"having 1 and keeping 3"), //留1,3
                                (vec![Card{mana:2, card_power:CardPower::Normal}, Card{mana:4, card_power:CardPower::Normal}],vec![Card{mana:2, card_power:CardPower::Normal}],"having 2 and keeping 4"), //留2,4
                            ];
        for pattern in kept_hand.into_iter()
        {
            let score1;
            let score2;

            let hand1 = match self.dealer.cards_to_position(pattern.0, CardLocation::InDeck, None)
            {
                Some(h)=>h,
                None=>continue,
            };
            let hand2 = match self.dealer.cards_to_position(pattern.1, CardLocation::InDeck, None)
            {
                Some(h)=>h,
                None=>continue,
            };

            let do_print = false;
            match self.start_simulation(hand1, do_print)
            {
                Some(s)=>score1=s,
                None=>continue,
            }

            match self.start_simulation(hand2, do_print)
            {
                Some(s)=>score2=s,
                None=>continue,
            }

            let score = score1 - score2;
            if score >= 0.000
            {
                println!("the value of {} is {}", pattern.2, format!("{:.2}",score).to_string().green());
            }
            else
            {
                println!("the value of {} is {}", pattern.2, format!("{:.2}",score).to_string().red());
            }

        }
    }
}

///do a command.
fn do_command(cmd:String, sim:&mut Simulator) -> CommandResult
{
    let mut cmd = cmd.split(' ').map(|x| x.trim().to_string()).collect::<Vec<String>>();
    match cmd.remove(0).as_str()
    {
        "help"=>
        {
            let mut file = File::open("help_text.txt").unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            println!("{}", contents);
        }
        "hand"=>
        {
            sim.reset();
            let mut hand = Vec::new();
            for word in cmd.iter()
            {
                match Card::create(word)
                {
                    Some(c)=>hand.push(c),
                    None=>return CommandResult::Err("failed to create card".to_string()),
                }
            }


            let order = match hand.len()
            {
                3 => PlayOrder::First,
                4 => PlayOrder::Second,
                _ => return CommandResult::Err("start hand should contain exactly 3 or 4 cards".to_string()),
            };

            println!("the hand is {:?}, going {:?}", hand, order);
            sim.set_play_order(order);
            let hand = match sim.dealer.cards_to_position(hand, CardLocation::InDeck, None)
            {
                Some(h)=>h,
                None=>return CommandResult::Err("card not in deck!".to_string()),
            };
            sim.solve_mull(hand);
        }
        "deck"=>
        {
            println!("the deck is :");
            println!("{:?}", sim.dealer);
            println!("the length is {}\n", sim.dealer.cards.len());

            println!("curve:");
            for mana_cost in 0..=10
            {
                let count = sim.dealer.cards.iter().filter(|x| x.mana == mana_cost).count();
                match count
                {
                    0 => println!("{:02}|", mana_cost),
                    _ => println!("{:02}|{} {}", mana_cost, "#".repeat(count).yellow(), count),
                }
            }
            println!();
        }
        "basic"=>
        {
            sim.sim_common_pattern();
        }
        "clear"=>
        {
            sim.clear();
            println!("deck cleared!");
        }
        "q"=>
        {
            return CommandResult::End;
        }
        "add"=>
        {
            for word in cmd.iter()
            {
                let result = sim.dealer.insert_cards(word);
                match result
                {
                    CommandResult::Err(_)=>return result,
                    _=>continue,
                };
            }
            sim.dealer.sort_deck();
        }
        "fill"=>
        {
            let result = sim.dealer.fill_deck();
            match result
            {
                CommandResult::Err(_)=>return result,
                _=>println!("the deck is filled with high-cost cards"),
            };
        }
        "hero"=>
        {
            match cmd.get(0)
            {
                Some(w)=>return sim.set_hero(w),
                None=>return CommandResult::Err("failed to set hero.\n
                                                dh for dh\n
                                                wr for warrior\n
                                                wl for warlock\n
                                                ma for mage\n
                                                pr for priest\n
                                                dr for druid\n
                                                sh for shaman\n
                                                hu for hunter\n
                                                pa for paladin\n
                                                ro for rogue\n".to_string()),
            }
        }
        "save"=>
        {
            let filename = match cmd.get(0)
            {
                Some(w)=>w,
                None=>"deck_file"
            };
            sim.dealer.save(filename.to_string());
            println!("deck saved!");
        }
        "load"=>
        {
            let filename = match cmd.get(0)
            {
                Some(w)=>w,
                None=>"deck_file"
            };
            return sim.dealer.load(filename.to_string());
        }
        "demo"=>
        {
            sim.play_order = PlayOrder::flip_the_coin();

            let start_hand_size;
            match sim.play_order
            {
                PlayOrder::First=>{println!("going first.");start_hand_size=3;},
                PlayOrder::Second=>{println!("going second.");start_hand_size=4;},
            }
            sim.reset();

            //draw cards
            for _ in 0..start_hand_size
            {
                sim.draw_card();
            }

            println!("the start hand is {:?}", sim.dealer.get_hand(&sim.dealer.card_location));
            let hand = sim.hand.clone();
            let mull = sim.solve_mull(sim.hand.clone());
            sim.reset();
            
            let keep = hand.into_iter().zip(mull.chars()).filter(|(_,mu)| mu==&'1').map(|(x,_) | x).collect::<Vec<usize>>();
            sim.set_start_hand(&keep);
            for i in 1..=10
            {
                sim.draw_card();
                let do_print = true;
                sim.play_a_turn(None, i, sim.max_search_depth, do_print);
            }
            sim.reset();
        }
        "test"=> //for test only
        {
        }
        _=>return CommandResult::Err("invalid command".to_string()),
    }
    CommandResult::Ok
}

fn read_config() -> Result<Simulator, Box<dyn Error>>
{

    let mut cycle_reps:i32 = i32::default();
    let mut maxturn:u8 = u8::default();
    let mut max_search_depth:u8 = u8::default();
    let mut play_card_bonus:i8 = i8::default();

    let mut is_cycle_reps_set = false;
    let mut is_maxturn_set = false;
    let mut is_max_search_depth_set = false;
    let mut is_play_card_bonus_set = false;


    let mut file = File::open("config.txt")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    for config in contents.trim().split('\n')
    {
        let config:Vec<&str> = config.trim().split(' ').collect();
        if config.len()!=2
        {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput,"")));
        }
        match config[0]
        {
            "cycle_reps"=>
            {
                is_cycle_reps_set = true;
                cycle_reps = config[1].parse()?;
            },
            "maxturn"=>
            {
                is_maxturn_set = true;
                maxturn = config[1].parse()?;
            },
            "max_search_depth"=>
            {
                is_max_search_depth_set = true;
                max_search_depth = config[1].parse()?;
            },
            "play_card_bonus"=>
            {
                is_play_card_bonus_set = true;
                play_card_bonus = config[1].parse()?;
            }
            _=>(),
        };
    }

    if is_cycle_reps_set && is_maxturn_set && is_max_search_depth_set && is_play_card_bonus_set
    {
        Ok(Simulator::new(cycle_reps, maxturn, max_search_depth, play_card_bonus))
    }
    else
    {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput,"")));
    }
}


fn main() {
    let mut sim = match read_config()
    {
        Ok(s) => s,
        Err(_) => 
        {
            println!("failed to read the config file");
            return;
        }
    };

    loop
    {
        let mut line = String::new();
        println!("Enter command: (enter help to get help)");
        std::io::stdin().read_line(&mut line).unwrap();
        match do_command(line.trim().to_string(), &mut sim)
        {
            CommandResult::Err(e)=>println!("{}", e),
            CommandResult::End=>break,
            CommandResult::Ok=>continue,
        }
    }
}

#[cfg(test)]
    mod tests {
    use super::*;

    #[test]
    fn normal_test()
    {
        let sim = Simulator::new(100, 6, 2, 1);
        let hand = vec![
                        Card{mana:2, card_power:CardPower::Normal},
                        Card{mana:8, card_power:CardPower::Normal},
                        Card{mana:8, card_power:CardPower::Normal},
                        // Card{mana:3, card_power:CardPower::Normal},
                        // Card{mana:2, card_power:CardPower::Normal},
                        // Card{mana:1, card_power:CardPower::Normal},
                    ];
        let all_plays = sim.get_all_play_patterns(hand, 9, 0);
        eprintln!("all_plays is {:?}", all_plays);
        panic!("a");
    }

    #[test]
    fn coin_test()
    {
        let sim = Simulator::new(100, 6, 2, 1);
        let hand = vec![
                        Card{mana:4, card_power:CardPower::Normal},
                        Card{mana:3, card_power:CardPower::Normal},
                        Card{mana:3, card_power:CardPower::Normal},
                        Card{mana:2, card_power:CardPower::Normal},
                        Card{mana:5, card_power:CardPower::Normal},
                        Card{mana:-1, card_power:CardPower::Normal},
                    ];
        let all_plays = sim.get_all_play_patterns(hand, 4, 0);
        eprintln!("all_plays is {:?}", all_plays);
        panic!("a");
    } 

    #[test]
    fn simple_test()
    {
        let sim = Simulator::new(100, 6, 2, 0);
        let hand = vec![
                        Card{mana:5, card_power:CardPower::Normal},
                        Card{mana:-1, card_power:CardPower::Normal},
                    ];
        let all_plays = sim.get_all_play_patterns(hand, 4, 0);
        eprintln!("all_plays is {:?}", all_plays);
        assert_eq!(vec![1,2].sort(),vec![2,1].sort());
        panic!("a");
    }

    #[test]
    fn future_score_test()
    {
        let mut sim = Simulator::new(100, 6, 2, 0);
        let hand = vec![
                        Card{mana:1, card_power:CardPower::Normal},
                        Card{mana:2, card_power:CardPower::Normal},
                        Card{mana:2, card_power:CardPower::Normal},
                        Card{mana:4, card_power:CardPower::Normal},
                    ];
        do_command("load".to_string(), &mut sim);
        sim.dealer.adjust_coin(PlayOrder::First);
        sim.play_a_hand(hand, 5, true);
        panic!("a");
    }

    #[test]
    fn different_card_power()
    {
        let mut sim = Simulator::new(100, 6, 2, 0);
        let hand = vec![
                        Card{mana:2, card_power:CardPower::Normal},
                        Card{mana:2, card_power:CardPower::Strong},
                        Card{mana:4, card_power:CardPower::Weak},
                        Card{mana:4, card_power:CardPower::Strong},
                    ];
        do_command("load card_power_test".to_string(), &mut sim);
        sim.dealer.adjust_coin(PlayOrder::First);
        sim.play_a_hand(hand, 6, true);
        panic!("a");
    }

}
