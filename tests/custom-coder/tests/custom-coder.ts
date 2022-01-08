//import * as anchor from '@project-serum/anchor';
//import { Program } from '@project-serum/anchor';
import * as anchor from '../../../ts';
import { Spl } from '../../../ts';

describe('custom-coder', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  it('Is initialized!', async () => {
		const program = Spl.token();
		console.log(program);
  });
});
