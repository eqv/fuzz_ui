template_html = `
<nav class="navbar navbar-expand-lg navbar-light bg-light">
		<a class="navbar-brand" href="#">IJON
			<i class="fas fa-user-astronaut"></i>
		</a>
		<button class="navbar-toggler" type="button" data-toggle="collapse" data-target="#navbarSupportedContent" aria-controls="navbarSupportedContent" aria-expanded="false" aria-label="Toggle navigation">
    <span class="navbar-toggler-icon"></span>
  </button>

  <div class="collapse navbar-collapse" id="navbarSupportedContent">
    <ul class="navbar-nav mr-auto">
      <!-- <li class="nav-item active"> <a class="nav-link" href="#">Dashboard <span class="sr-only">(current)</span></a> </li>  -->
      <li :class='["nav-item", {active: current==="dash"}]'> <a class="nav-link" href="/"><i class="fas fa-chart-area"></i> Dashboard</a> </li>
			<li :class='["nav-item", {active: current==="src"}]'> <a class="nav-link" href="/static/src.html"><i class="fas fa-align-left"></i> Coverage</a> </li>
			<li :class='["nav-item", {active: current==="queue"}]'> <a class="nav-link" href="/static/queue.html"><i class="fas fa-database"></i> Queue</a> </li>
    </ul>

  </div>
</nav>
`

Vue.component('ijon-nav-bar', {
		props: ['current'],
		template: template_html,
		methods:{
		}
	})
