import { TransactionInstruction } from '@solana/web3.js';
import { Program } from './program';

export default class RequestBuilder {

	private _program: Program;
	private _accounts: any;
	private _ns: RequestNamespace;
	private _ixs: TransactionInstruction[];
	private _clusterUrl?: string;
	private _signers: Account[];

	constructor(program: Program, ns: RequestNamespace) {
		this._program = program;
		this._ns = ns;
		this._accounts = [];
		this._ixs = [];
	}

	public accounts(accs: Object): RequestBuilder {
		this._accounts = accs;
		return this;
	}

	public instruction(ix: TransactionInstruction): RequestBuilder {
		this._ixs.push(ix);
		return this;
	}

	public cluster(url: string): RequestBuilder {
		this._clusterUrl = url;
		return this;
	}

	public signer(account: Account): RequestBuilder {
		this._signers.push(account);
		return this;
	}

	// Returns tx signature.
	public send(): string {
		return '';
	}
}

type RequestNamespace = Global | State | Interface;
type Global = { global: {} };
type State = { state: {} };
type Interface = { interface: {} };
