var devices = [];

function reloadModelGraph() {
	$.getJSON('/api/v1/popular/carrier/90', function(data) {
		devices = data.result;
		doPie(document.getElementById('chart_devices'), devices);
	});

}

function doPie(el, data) {
	var labels = data.map(function(a) {return a._id});
	var counts = data.map(function(a) {return a.total});
	new Chartist.Pie('.ct-chart', {labels: labels, series:counts})
}

$(document).ready(reloadModelGraph);	

