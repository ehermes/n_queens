extern crate rand;
extern crate pbr;

use rand::distributions::{IndependentSample, Range};
use pbr::ProgressBar;

// N = size of the board
const N: usize = 8;
// highest temperature for parallel tempering
const T_HIGH: f32 = 2f32;
// lowest temperature for parallel tempering
const T_LOW: f32 = 0.01f32;
// Number of images
const NT: usize = 20;
// number of moves to give up finding a solution after
const MAXMOVES: usize = 100000000;
// Probability of swapping two boards
const PSWAP: f32 = 1f32 / NT as f32;

#[derive(Copy, Clone)]
struct Queen {
    x: usize,
    y: usize,
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

struct Board {
    queens: [Queen; N],
    e: isize,
}

impl Board {
    fn new(queens: [Queen; N]) -> Board {
        let mut board = Board{ queens: queens, e: 0 };
        board.e = board.calc_e();
        return board;
    }

    fn calc_e(&mut self) -> isize {
        let mut e = 0;
        for i in 1..N {
            for j in 0..i {
                if self.queens[i].clash(&self.queens[j]) {
                    e += 1;
                }
            }
        }
        return e;
    }


    fn de_move(&self, qn: usize, xnew: usize, ynew: usize) -> isize {
        let mut de: isize = 0;
        let newqueen = Queen { x: xnew, y: ynew };
        for i in 0..N {
            if i == qn { continue };
            if self.queens[i].clash(&self.queens[qn]) {
                de -= 1;
            };
            if self.queens[i].clash(&newqueen) {
                de += 1;
            };
        }
        return de;
    }

    fn move_queen(&mut self, qn: usize, x: usize, y: usize) {
        if self.queens[qn].x == x && self.queens[qn].y == y {
            return
        };
        self.e += self.de_move(qn, x, y);
        self.queens[qn] = Queen { x: x, y: y };
    }

    fn print_board(&self) {
        let mut barray = [[0usize; N]; N];
        for queen in self.queens.iter() {
            barray[queen.x][queen.y] += 1;
        }
        print!{"╔═"};
        for _ in 1..N {
            print!{"╤═"};
        }
        print!{"╗\n"};
        for i in 0..N {
            print!{"║{}", barray[i][0]};
            for j in 1..N {
                print!{"│{}", barray[i][j]};
            }
            print!{"║\n"};
            if i != N - 1 {
                print!{"╟─"};
                for _ in 1..N {
                    print!{"┼─"};
                }
                print!{"╢\n"};
            }
        }
        print!{"╚═"};
        for _ in 1..N {
            print!{"╧═"};
        }
        print!{"╝\n"};
    }
}


fn main() {
    let mut totsteps: usize = 0;
    // Create RNG stuff here and pass it to run, so we don't
    // re-initialize RNG for every solution. This might not be
    // necessary.
    let mut rng = rand::weak_rng();
    let nrange = Range::new(0, N);
    let ntrange = Range::new(0, NT);
    let probrange = Range::new(0f32, 1f32);

    // Number of minimizations to perform
    let trials = 200000;

    let mut pb = ProgressBar::new(trials);
    for i in 0..trials {
        totsteps += run(&nrange, &ntrange, &probrange, &mut rng);
        pb.message(format!("Average number of steps: {} | ", totsteps as f32/((i+1) as f32)).as_str());
        pb.inc();
    };
    pb.finish_println("Done!\n");
}

fn run(nrange: &Range<usize>, ntrange: &Range<usize>, probrange: &Range<f32>, rng: &mut rand::XorShiftRng) -> usize {

    // Temperature increase factor between each adjacent image
    let tfact: f32 = (T_HIGH / T_LOW).powf((NT as f32).powi(-1));
    // Vec of beta values
    let mut bs: Vec<f32> = Vec::new();

    let mut boards: Vec<Board> = Vec::new();

    // Create NT NxN boards, each with N randomly placed queens
    for i in 0..NT {
        bs.push((T_LOW * tfact.powi(i as i32)).powi(-1));
        let mut board = Board::new([Queen { x: 0, y: 0 }; N]);
        
        for j in 0..N {
            let x = nrange.ind_sample(rng);
            let y = nrange.ind_sample(rng);
            board.move_queen(j, x, y);
        };
        boards.push(board);
    };

    for j in 1..MAXMOVES {
        // Pick a board to work with
        let bn = ntrange.ind_sample(rng);

        // Check to see if we're switching boards
        if probrange.ind_sample(rng) < PSWAP {
            // If we picked the last board and are swapping boards, just do
            // nothing and continue with the next loop. This effectively lowers
            // our probability of swapping boards, but it makes the code a lot
            // simpler.
            if bn == NT - 1 { continue };
            let de = boards[bn + 1].e - boards[bn].e;
            let db = bs[bn] - bs[bn + 1];
            if (de as f32 * db < 0f32) || (probrange.ind_sample(rng) < (-de as f32 * db).exp()) {
                boards.swap(bn, bn+1);
            };
            continue;
        };

        // Pick a random queen to move and a new position
        let qn = nrange.ind_sample(rng);
        let x = nrange.ind_sample(rng);
        let y = nrange.ind_sample(rng);
        let de = boards[bn].de_move(qn, x, y);
 
        // Metropolis-Hastings algorithm. If de <= 0, accept the move.
        // Otherwise, accept the move with probability given by e^(-dE/kT)
        // (so, bigger de => less likely to accept move). Here we replace
        // 1/kT with B.
        // If we accept the move, replace the old queen with the new one
        // and update the total system energy.

        if (de <= 0) || (probrange.ind_sample(rng) < (-de as f32 * bs[bn]).exp()) {
            boards[bn].move_queen(qn, x, y);

            // If the energy is 0, we're done.
            if boards[bn].e == 0 {
                //boards[bn].print_board();
                return j;
            };
        };
    };
    // Rather than actually handling the failure case, just return MAXMOVES.
    // This should not affect the results unless the board is very big.
    return MAXMOVES;
}
