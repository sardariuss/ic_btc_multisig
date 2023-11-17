import '../App.css';

const Title = () => {

  const title = "Multi-Subnet Bitcoin Wallet";

  return (
    <div className="stack title" style={{"--stacks": "3"} as React.CSSProperties} >
      <span style={{"--index": "0"} as React.CSSProperties}>{title}</span>
      <span style={{"--index": "1"} as React.CSSProperties}>{title}</span>
      <span style={{"--index": "2"} as React.CSSProperties}>{title}</span>
    </div>
  )
}

export default Title;