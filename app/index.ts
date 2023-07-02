
import('../pkg').then((ltlEngine) => {

  const len = 1_000;

  ltlEngine.Config.randomize();
  document['board'] = ltlEngine.Board;
  document['config'] = ltlEngine.Config;
  new ltlEngine.Board(BigInt(len.toString()), ltlEngine.Config.randomize());
  console.log(1);
})

// const arr = Array(len).fill(Array(len).fill(0))
// for (let x = 0; x < len; x++) {
//   for (let y = 0; y < len; y++) {
//     arr[x][y] += 1
//   }
// }
// console.log(arr);



export { }
