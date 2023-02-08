template_html = `
<div id='code' class="col-md-9">
	<pre v-if="lines">
    <div v-for="line in lines"> <span class="line_span" v-html="line"></span> </div>
    </pre>
</div>
`

Vue.component('ijon-hex-view', {
		template: template_html,
		props: ['hex_data'],
		methods: {
            ascii_char: function(c){
                if(c > 32 && c < 127){return String.fromCharCode(c)}
                return "."
            },

            hex_line: function(addr,dat){
                var line = (addr).toString(16).padStart(8, '0')+"    ";
                var ascii = "";
                for(c in dat){
                    c=dat[c]
                    line+=c.toString(16).padStart(2,"0")+" ";
                    ascii += this.ascii_char(c);
                }
                return line.padEnd(8+4+16*3," ")+"|"+ascii.padEnd(16," ")+"|";
            }
        },
        computed: {
            lines: function() {
                var res = [];
                for(var i=0; i< Math.ceil(this.hex_data.length/16); i++){
                    res.push( this.hex_line(i*16, this.hex_data.slice(i*16,i*16+16)) );
                }
                return res;
            }
        }
	})
