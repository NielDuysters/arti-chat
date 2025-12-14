import React from "react";
import { ActionType } from "./ActionType";
import "./Action.scss";

export default function Action({label, description, actionType, onClick, success}) {
    
    const renderButton = (actionType, onClick) => {

        switch (actionType) {
            case ActionType.Delete:
                const buttonImage = (success) => {
                    if (success === null) {
                        return (
                            <img
                                className="action__button__image"
                                src="/src/assets/delete.png"
                            />
                        )
                    }

                    if (success === false) {
                        return (
                            <img
                                className="form__submit-btn__status"
                                alt="Failed"
                                src="/src/assets/button-failed.png"
                            />
                        );
                    }

                    return (
                        <img
                            className="form__submit-btn__status"
                            alt="Success"
                            src="/src/assets/button-success.png"
                        />
                    );
                };

                return (
                    <button
                        className="action__button action__button--delete"
                        onClick={onClick}
                    >
                        { buttonImage(success) }
                        Delete
                    </button>
                )
        }
    };

    return (
        <div className="action">
            <span className="action__label">{label}</span>
            <div className="action__content">
                <span className="action__description">{description}</span>
                { renderButton(actionType, onClick) }
            </div>
        </div>
    );
}

