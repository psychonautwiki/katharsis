class Events {
	constructor() {
		this._events = {};
	}

	on(evt, fn) {
		this._events[evt] = fn;
	}

	emit(evt, ...args) {
		this._events[evt](...args);
	}
}

class Katharsis extends Events {
	constructor() {
		super();

		this._constants = {
			endpoint: '/katharsis.json',

			grace: 200,
			max: 15
		};

		this._internal = {
			_tt: 0,
			_retries: 0
		};

		this._container = document.getElementsByClassName('rx-katharsis-container')[0];

		this._dashboard = document.createElement('span');
		this._stackdriver = document.createElement('span');

		this._container.appendChild(this._dashboard);
		this._container.appendChild(this._stackdriver);

		this.on('update', data => this._render(data));
	}

	_ingestError(err) {
		if ('Raven' in window && 'captureException' in Raven) {
			/* send to sentry */
			Raven.captureException(err);
		}
	}

	_loadLoop() {
		try {
			const xhr = new XMLHttpRequest();

			xhr.open("GET", this._constants.endpoint, true);

			xhr.setRequestHeader('Content-Type', 'application/json');

			xhr.onreadystatechange = () => {
				if (xhr.readyState !== 4 || xhr.status !== 200) {
					return;
				}

				try {
					this.emit('update', JSON.parse(xhr.responseText));

					setTimeout((() => this._loadLoop()), 2000);
				} catch (e) {
					this._ingestError(e);
				}
			};

			xhr.send();
		} catch (e) {
			this._ingestError(e);
		}
	}

	_buildItemPanel(set) {
		const panel = document.createElement('div');
		panel['className'] = 'flex-panel row-reverse';

		const buildPanelItem = (value, label) => {
			const rootColumn = document.createElement('div');
			rootColumn['className'] = 'flex-column very-very-wide';

			const item = document.createElement('div');
			item['className'] = 'panel radius';

			const head = document.createElement('h3');
			head['className'] = 'panel-header';

			const interior = document.createElement('span');
			interior['className'] = 'mw-headline';

			const valueItem = document.createElement('span');
			valueItem['className'] = 'value-item';
			valueItem['innerText'] = value;

			const valueLabel = document.createElement('span');
			valueLabel['className'] = 'value-label';
			valueLabel['innerText'] = label;

			interior['appendChild'](valueItem);
			interior['appendChild'](valueLabel);

			head['appendChild'](interior);
			item['appendChild'](head);
			rootColumn['appendChild'](item);

			return rootColumn;
		};

		set.forEach(([item, label]) =>
			panel['appendChild'](buildPanelItem(item, label))
		);

		return panel;
	}

	_render(data) {
		const set = [
			[data['total']['total'], 'total users'],
			[data['total']['new'], 'new users'],
			[data['total']['unique'], 'unique users']
		];

		const panelContainer = this._buildItemPanel(set);

		while (this._dashboard['firstChild']) {
			this._dashboard['removeChild'](this._dashboard['firstChild']);
		}

		this._dashboard.appendChild(panelContainer);
	}

	_renderDashboard() {
		const frame = document.createElement('iframe');

		frame.src = 'https://public.google.stackdriver.com/public/chart/qxc0ou9lkEvcRVl4?drawMode=color&showLegend=true&theme=light';
		frame.height = 400;
		frame.scrolling = false;
		frame.seamless = 'seamless';

		this._stackdriver.appendChild(frame);
	}

	init() {
		this._loadLoop();
		this._renderDashboard();
	}
}

if (document.getElementsByClassName('rx-katharsis-container').length > 0) {
	let katharsis = new Katharsis();

	katharsis.init();
}