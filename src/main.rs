extern crate rand;
extern crate pbr;

//use std::collections::HashMap;
use rand::distributions::{IndependentSample, Range};
use pbr::ProgressBar;

// N = size of the board
const N: usize = 8;
// B is beta, 1/kB T
const B: f32 = 1f32/0.165f32;
// number of moves to give up finding a solution after
const MAXMOVES: usize = 1000000;

struct Queen {
    x: isize,
    y: isize,
}

impl Queen {
    fn clash(&self, other: &Queen) -> bool {
        // two queens are in the same column
        if self.x == other.x { return true };
        // two queens are in the same row
        if self.y == other.y { return true };
        // two queens are in the same \
        if self.x + self.y == other.x + other.y { return true };
        // two queens are in the same /
        if self.x - self.y == other.x - other.y { return true };
        // The queens are not clashing.
        return false;
    }
}



fn main() {
    let mut totsteps: usize = 0;
    // Create RNG stuff here and pass it to run, so we don't
    // re-initialize RNG for every solution. This might not be
    // necessary.
    let mut rng = rand::weak_rng();
    let between = Range::new(0, N);
    let probrange = Range::new(0f32, 1f32);
//    let mut lookup = de_prob_lookup();

    // Number of minimizations to perform
    let trials = 1000000;

    let mut pb = ProgressBar::new(trials);
    for i in 0..trials {
        totsteps += run(&between, &probrange, &mut rng);
        pb.message(format!("Average number of steps: {} | ", totsteps as f32/((i+1) as f32)).as_str());
        pb.inc();
    };
    pb.finish_println("Done!");
}

//fn de_prob_lookup() -> HashMap<&isize, &f32> {
//    let mut lookup = HashMap::new();
//    let max_clashes = (N * (N - 1)) / 2;
//    for i in 1..max_clashes {
//        lookup.insert(i, (i as f32 * B).exp());
//    };
//    lookup
//}

fn run(between: &Range<usize>, probrange: &Range<f32>, rng: &mut rand::XorShiftRng) -> usize {
    // Create a Vec of randomly placed Queens
    let mut queens = Vec::new();
    for _ in 0..N {
        let x = between.ind_sample(rng);
        let y = between.ind_sample(rng);
        queens.push(Queen { x: x as isize, y: y as isize });
    };

    // "Energy" of the system. A solution will have e==0.
    let mut e: isize = 0;

    // Calculate initial energy for random configuration
    for i in 1..N {
        for j in 0..i {
            if queens[i].clash(&queens[j]) {
                e += 1;
            }
        }
    };

    // Change of energy associated with a trial move
    let mut de: isize;

    for j in 1..MAXMOVES {
        de = 0;
        // Pick a random queen to move and a new position
        let qn = between.ind_sample(rng);
        let x = between.ind_sample(rng);
        let y = between.ind_sample(rng);
        let newqueen = Queen { x: x as isize, y: y as isize };

        // Calculate de by looking at all clashes for new and old position
        for i in 0..N {
            if i == qn { continue };
            if queens[qn].clash(&queens[i]) {
                de -= 1;
            };
            if newqueen.clash(&queens[i]) {
                de += 1;
            };
        };
        
        // Metropolis-Hastings algorithm. If de <= 0, accept the move.
        // Otherwise, accept the move with probability given by e^(-dE/kT)
        // (so, bigger de => less likely to accept move). Here we replace
        // 1/kT with B.
        // If we accept the move, replace the old queen with the new one
        // and update the total system energy.
        if probrange.ind_sample(rng) < (-de as f32*B).exp() {
            queens[qn] = newqueen;
            e += de;

            // If the energy is 0, we're done.
            if e == 0 {
                return j;
            };
        };
    };
    // Rather than actually handling the failure case, just return MAXMOVES.
    // This should not affect the results unless the board is very big.
    return MAXMOVES;
}
