import { useNavigate } from 'react-router-dom'
import { getTokenPayload, logout } from '../auth'

export function Navbar() {
  const navigate = useNavigate()
  const user = getTokenPayload()

  const handleLogout = () => {
    logout()
    navigate('/login')
  }

  return (
    <div className="navbar bg-base-100 border-b border-base-200 px-4">
      <div className="navbar-start">
        <span className="text-lg font-bold text-primary">Coworking</span>
      </div>
      <div className="navbar-end gap-3">
        {user && (
          <span className="text-sm text-base-content/60 hidden sm:block">
            {user.first_name}
          </span>
        )}
        <button className="btn btn-ghost btn-sm" onClick={handleLogout}>
          Sign out
        </button>
      </div>
    </div>
  )
}
