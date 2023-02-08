//template_html = `
//<div class="codeview">
//	<pre v-if="code">
//<div 
//		v-for="line in code" 
//			v-bind:id="'l'+line.index" 
//			v-bind:class="'line '+line.class"
//			@click="set_line(line.index)"
//			:class='[{"selected": selected_line === line.index}, "line", line.klass]'
//	><span class="line_span" v-html="line.content"></span></div></pre>
//</div>
//`

template_html = `
<div class="codeview">
	<pre id=codeview-pre />
</div>
`

Vue.component('ijon-code-view', {
		template: template_html,
		props: ['code', 'src_file', 'selected_line', 'file_infos'],
		methods: {
			set_line: function(line){
				console.log("set line",line);
				this.$emit("select-line",this.src_file,line);
			}
		},
		watch: {
			code: function(){
				console.log("RECALC CODE");
				let c = this.code;
				let res = ""
				for(line in c){
					let ll = c[line];
					var klass = ll.klass;
					if(ll.index === this.selected_line){klass+=" selected";}
					res+="<div id=l"+ll.index+" class=\"line "+klass+"\" style=\"width: 1000px;\" onclick=\"document.code_view_elem.set_line("+ll.index+")\"><span class='line_span'>"+ll.content+"</span></div>"
				}
				document.getElementById("codeview-pre").innerHTML = res;
			},
			selected_line: function(new_line, old_line){
				console.log("update selected line",old_line, new_line);
				document.getElementById("l"+old_line).classList.remove('selected');
				document.getElementById("l"+new_line).classList.add('selected');
			}
		},
		created: function(){
			document.code_view_elem = this
		}
	})